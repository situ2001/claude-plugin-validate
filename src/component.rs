use crate::{ComponentValidation, ValidationIssue};

pub fn validate_component_markdown(
    file_path: &str,
    content: &str,
    file_type: &str,
) -> ComponentValidation {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let frontmatter = extract_frontmatter(content);
    let Some(frontmatter_text) = frontmatter else {
        warnings.push(issue(
            "frontmatter",
            "missing_frontmatter",
            "No frontmatter block found. Add YAML frontmatter between --- delimiters at the top of the file to set description and other metadata.",
        ));
        return ComponentValidation {
            path: file_path.to_string(),
            errors,
            warnings,
        };
    };

    let parsed: serde_yaml::Value = match serde_yaml::from_str(frontmatter_text) {
        Ok(v) => v,
        Err(err) => {
            errors.push(issue(
                "frontmatter",
                "invalid_yaml",
                &format!(
                    "YAML frontmatter failed to parse: {err}. At runtime this {file_type} loads with empty metadata (all frontmatter fields silently dropped)."
                ),
            ));
            return ComponentValidation {
                path: file_path.to_string(),
                errors,
                warnings,
            };
        }
    };

    let Some(map) = parsed.as_mapping() else {
        errors.push(issue(
            "frontmatter",
            "invalid_type",
            "Frontmatter must be a YAML mapping (key: value pairs).",
        ));
        return ComponentValidation {
            path: file_path.to_string(),
            errors,
            warnings,
        };
    };

    match get_yaml_value(map, "description") {
        None => warnings.push(issue(
            "description",
            "missing_description",
            &format!(
                "No description in frontmatter. A description helps users and Claude understand when to use this {file_type}."
            ),
        )),
        Some(v) => {
            if !is_yaml_scalar(v) {
                errors.push(issue(
                    "description",
                    "invalid_type",
                    "description must be a scalar (string/number/bool/null).",
                ));
            }
        }
    }

    if let Some(v) = get_yaml_value(map, "name")
        && !is_yaml_null(v)
        && !matches!(v, serde_yaml::Value::String(_))
    {
        errors.push(issue("name", "invalid_type", "name must be a string."));
    }

    if let Some(v) = get_yaml_value(map, "allowed-tools") {
        let ok = match v {
            serde_yaml::Value::String(_) => true,
            serde_yaml::Value::Sequence(seq) => seq
                .iter()
                .all(|item| matches!(item, serde_yaml::Value::String(_))),
            serde_yaml::Value::Null => true,
            _ => false,
        };
        if !ok {
            errors.push(issue(
                "allowed-tools",
                "invalid_type",
                "allowed-tools must be a string or array of strings.",
            ));
        }
    }

    if let Some(v) = get_yaml_value(map, "shell")
        && !is_yaml_null(v)
    {
        match v {
            serde_yaml::Value::String(s) => {
                let norm = s.trim().to_lowercase();
                if norm != "bash" && norm != "powershell" {
                    errors.push(issue(
                        "shell",
                        "invalid_enum_value",
                        &format!("shell must be 'bash' or 'powershell', got '{s}'."),
                    ));
                }
            }
            _ => errors.push(issue("shell", "invalid_type", "shell must be a string.")),
        }
    }

    ComponentValidation {
        path: file_path.to_string(),
        errors,
        warnings,
    }
}

fn extract_frontmatter(content: &str) -> Option<&str> {
    if !content.starts_with("---\n") {
        return None;
    }
    let rest = &content[4..];
    let end = rest.find("\n---\n")?;
    Some(&rest[..end])
}

fn get_yaml_value<'a>(map: &'a serde_yaml::Mapping, key: &str) -> Option<&'a serde_yaml::Value> {
    map.get(serde_yaml::Value::String(key.to_string()))
}

fn is_yaml_scalar(v: &serde_yaml::Value) -> bool {
    matches!(
        v,
        serde_yaml::Value::Null
            | serde_yaml::Value::Bool(_)
            | serde_yaml::Value::Number(_)
            | serde_yaml::Value::String(_)
    )
}

fn is_yaml_null(v: &serde_yaml::Value) -> bool {
    matches!(v, serde_yaml::Value::Null)
}

fn issue(path: &str, code: &str, message: &str) -> ValidationIssue {
    ValidationIssue {
        path: path.to_string(),
        code: code.to_string(),
        message: message.to_string(),
    }
}
