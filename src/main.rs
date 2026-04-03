use anyhow::{Context, Result};
use cc_plugin_validator::{
    ComponentValidation, ValidationIssue, ValidationResult, validate_component_markdown,
    validate_plugin_manifest,
};
use clap::{Parser, ValueEnum};
use owo_colors::{OwoColorize, set_override};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "claude-plugin-validate")]
#[command(about = "Validate plugin.json manifests and markdown component frontmatter")]
struct Cli {
    /// Target file or directory
    #[arg(default_value = ".")]
    target: String,

    /// Output in JSON
    #[arg(long)]
    json: bool,

    /// Validate everything, including markdown frontmatter in commands/agents/skills and hooks/hooks.json
    #[arg(long)]
    all: bool,

    /// Colorize output: auto, always, never
    #[arg(long, value_enum, default_value_t = ColorMode::Auto)]
    color: ColorMode,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Serialize)]
struct Row {
    file: String,
    ok: bool,
    issues: Vec<JsonIssue>,
    warnings: Vec<JsonIssue>,
}

#[derive(Debug, Serialize)]
struct JsonIssue {
    path: String,
    code: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct JsonOutput {
    target: String,
    total: usize,
    failed: usize,
    rows: Vec<Row>,
}

fn collect_plugin_json_files(target: &Path) -> Result<Vec<PathBuf>> {
    if target.is_file() {
        let is_plugin_json = target
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s == "plugin.json")
            .unwrap_or(false);
        if !is_plugin_json {
            anyhow::bail!("Expected plugin.json, got {}", target.display());
        }
        return Ok(vec![target.to_path_buf()]);
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if entry.file_type().is_dir() {
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if name == ".git" || name == "node_modules" {
                continue;
            }
        }

        if entry.file_type().is_file() {
            let is_plugin_json = path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s == "plugin.json")
                .unwrap_or(false);
            if is_plugin_json {
                files.push(path.to_path_buf());
            }
        }
    }

    files.sort();
    Ok(files)
}

fn issue_to_json(issue: &ValidationIssue) -> JsonIssue {
    JsonIssue {
        path: issue.path.clone(),
        code: issue.code.clone(),
        message: issue.message.clone(),
    }
}

fn push_component_result(rows: &mut Vec<Row>, result: ComponentValidation) {
    let ok = result.errors.is_empty();
    if ok && result.warnings.is_empty() {
        return;
    }

    rows.push(Row {
        file: result.path,
        ok,
        issues: result.errors.iter().map(issue_to_json).collect(),
        warnings: result.warnings.iter().map(issue_to_json).collect(),
    });
}

fn collect_markdown_files(plugin_dir: &Path, subdir: &str, skill_mode: bool) -> Vec<PathBuf> {
    let dir = plugin_dir.join(subdir);
    if !dir.exists() {
        return Vec::new();
    }

    let mut out = Vec::new();

    if skill_mode {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    out.push(p.join("SKILL.md"));
                }
            }
        }
        return out;
    }

    for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if entry.file_type().is_file()
            && p.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.to_ascii_lowercase().ends_with(".md"))
                .unwrap_or(false)
        {
            out.push(p.to_path_buf());
        }
    }

    out
}

fn validate_plugin_contents(plugin_dir: &Path, rows: &mut Vec<Row>) {
    let groups = [
        ("skills", "skill", true),
        ("agents", "agent", false),
        ("commands", "command", false),
    ];

    for (subdir, kind, skill_mode) in groups {
        let files = collect_markdown_files(plugin_dir, subdir, skill_mode);
        for file in files {
            match fs::read_to_string(&file) {
                Ok(content) => {
                    let result =
                        validate_component_markdown(&file.display().to_string(), &content, kind);
                    push_component_result(rows, result);
                }
                Err(err) => {
                    rows.push(Row {
                        file: file.display().to_string(),
                        ok: false,
                        issues: vec![JsonIssue {
                            path: "file".to_string(),
                            code: "io_error".to_string(),
                            message: format!("Failed to read markdown file: {err}"),
                        }],
                        warnings: vec![],
                    });
                }
            }
        }
    }

    let hooks_path = plugin_dir.join("hooks").join("hooks.json");
    if hooks_path.exists() {
        match fs::read_to_string(&hooks_path) {
            Ok(content) => {
                let row = match serde_json::from_str::<Value>(&content) {
                    Ok(json) => {
                        let hooks_value = json
                            .as_object()
                            .and_then(|m| m.get("hooks"))
                            .cloned()
                            .unwrap_or_else(|| Value::Object(Default::default()));

                        let fake = serde_json::json!({
                            "name": "tmp-plugin",
                            "hooks": hooks_value
                        });

                        match validate_plugin_manifest(fake) {
                            ValidationResult::Success { .. } => Row {
                                file: hooks_path.display().to_string(),
                                ok: true,
                                issues: vec![],
                                warnings: vec![],
                            },
                            ValidationResult::Failure { issues } => Row {
                                file: hooks_path.display().to_string(),
                                ok: false,
                                issues: issues.iter().map(issue_to_json).collect(),
                                warnings: vec![],
                            },
                        }
                    }
                    Err(err) => Row {
                        file: hooks_path.display().to_string(),
                        ok: false,
                        issues: vec![JsonIssue {
                            path: "json".to_string(),
                            code: "invalid_json".to_string(),
                            message: format!(
                                "Invalid JSON syntax: {err}. At runtime this breaks the entire plugin load."
                            ),
                        }],
                        warnings: vec![],
                    },
                };
                if !row.ok || !row.warnings.is_empty() {
                    rows.push(row);
                }
            }
            Err(err) => rows.push(Row {
                file: hooks_path.display().to_string(),
                ok: false,
                issues: vec![JsonIssue {
                    path: "file".to_string(),
                    code: "io_error".to_string(),
                    message: format!("Failed to read file: {err}"),
                }],
                warnings: vec![],
            }),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let target = PathBuf::from(&cli.target);
    let files = collect_plugin_json_files(&target)?;

    let mut rows: Vec<Row> = Vec::new();

    for file in files {
        let content = fs::read_to_string(&file)
            .with_context(|| format!("failed to read {}", file.display()))?;

        let row = match serde_json::from_str::<Value>(&content) {
            Ok(json) => match validate_plugin_manifest(json) {
                ValidationResult::Success { .. } => Row {
                    file: file.display().to_string(),
                    ok: true,
                    issues: vec![],
                    warnings: vec![],
                },
                ValidationResult::Failure { issues } => Row {
                    file: file.display().to_string(),
                    ok: false,
                    issues: issues.iter().map(issue_to_json).collect(),
                    warnings: vec![],
                },
            },
            Err(err) => Row {
                file: file.display().to_string(),
                ok: false,
                issues: vec![JsonIssue {
                    path: "<root>".to_string(),
                    code: "invalid_json".to_string(),
                    message: err.to_string(),
                }],
                warnings: vec![],
            },
        };

        if cli.all {
            let plugin_dir = file
                .parent()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            validate_plugin_contents(&plugin_dir, &mut rows);
        }

        rows.push(row);
    }

    rows.sort_by(|a, b| a.file.cmp(&b.file));

    let failed = rows.iter().filter(|r| !r.ok).count();

    if cli.json {
        let out = JsonOutput {
            target: fs::canonicalize(&target)
                .unwrap_or(target)
                .display()
                .to_string(),
            total: rows.len(),
            failed,
            rows,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        match cli.color {
            ColorMode::Auto => {}
            ColorMode::Always => set_override(true),
            ColorMode::Never => set_override(false),
        }

        let ok_label = "OK".green().bold().to_string();
        let err_label = "ERR".red().bold().to_string();

        for row in &rows {
            if row.ok {
                println!("{ok_label}  {}", row.file);
            } else {
                println!("{err_label} {}", row.file);
                for issue in &row.issues {
                    println!("  - {}: {}", issue.path, issue.message);
                }
            }
            for warning in &row.warnings {
                println!("WARN {}", row.file);
                println!("  - {}: {}", warning.path, warning.message);
            }
        }
        println!("\nSummary: {}/{} valid", rows.len() - failed, rows.len());
    }

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
