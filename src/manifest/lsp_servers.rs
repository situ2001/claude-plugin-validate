use super::common::{is_rel_json_path, obj_get, push_issue, validate_string_map};
use crate::ValidationIssue;
use serde_json::{Map, Value};

pub(super) fn validate_lsp_servers_field(
    root: &Map<String, Value>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(value) = obj_get(root, "lspServers") else {
        return;
    };

    validate_lsp_servers_value("lspServers", value, issues);
}

fn validate_lsp_servers_value(path: &str, value: &Value, issues: &mut Vec<ValidationIssue>) {
    if let Some(s) = value.as_str() {
        if !is_rel_json_path(s) {
            push_issue(
                issues,
                path,
                "invalid_string",
                "Path must start with ./ and end with .json",
            );
        }
        return;
    }

    if let Some(arr) = value.as_array() {
        for (idx, item) in arr.iter().enumerate() {
            validate_lsp_servers_value(&format!("{path}.{idx}"), item, issues);
        }
        return;
    }

    let Some(map) = value.as_object() else {
        push_issue(
            issues,
            path,
            "invalid_type",
            "lspServers must be path | map | array",
        );
        return;
    };

    for (server_name, server_cfg) in map {
        validate_lsp_server_config(&format!("{path}.{server_name}"), server_cfg, issues);
    }
}

fn validate_lsp_server_config(path: &str, value: &Value, issues: &mut Vec<ValidationIssue>) {
    let Some(obj) = value.as_object() else {
        push_issue(
            issues,
            path,
            "invalid_type",
            "LSP server config must be an object",
        );
        return;
    };

    match obj.get("command").and_then(Value::as_str) {
        Some(cmd) if !cmd.is_empty() => {
            if cmd.contains(' ') && !cmd.starts_with('/') {
                push_issue(
                    issues,
                    &format!("{path}.command"),
                    "custom",
                    "Command should not contain spaces. Use args for arguments.",
                );
            }
        }
        _ => push_issue(
            issues,
            &format!("{path}.command"),
            "missing_required",
            "command must be a non-empty string",
        ),
    }

    if let Some(args) = obj.get("args") {
        let Some(arr) = args.as_array() else {
            push_issue(
                issues,
                &format!("{path}.args"),
                "invalid_type",
                "args must be an array of non-empty strings",
            );
            return;
        };

        for (idx, item) in arr.iter().enumerate() {
            match item.as_str() {
                Some(s) if !s.is_empty() => {}
                _ => push_issue(
                    issues,
                    &format!("{path}.args.{idx}"),
                    "invalid_type",
                    "arg must be a non-empty string",
                ),
            }
        }
    }

    match obj.get("extensionToLanguage").and_then(Value::as_object) {
        Some(ext_map) if !ext_map.is_empty() => {
            for (ext, lang) in ext_map {
                if !ext.starts_with('.') {
                    push_issue(
                        issues,
                        &format!("{path}.extensionToLanguage.{ext}"),
                        "invalid_string",
                        "Extension key must start with dot",
                    );
                }
                if lang.as_str().map(|s| s.is_empty()).unwrap_or(true) {
                    push_issue(
                        issues,
                        &format!("{path}.extensionToLanguage.{ext}"),
                        "invalid_type",
                        "Language must be a non-empty string",
                    );
                }
            }
        }
        _ => push_issue(
            issues,
            &format!("{path}.extensionToLanguage"),
            "missing_required",
            "extensionToLanguage must be a non-empty map",
        ),
    }

    if let Some(transport) = obj.get("transport") {
        match transport.as_str() {
            Some("stdio") | Some("socket") => {}
            Some(_) => push_issue(
                issues,
                &format!("{path}.transport"),
                "invalid_enum_value",
                "transport must be stdio or socket",
            ),
            None => push_issue(
                issues,
                &format!("{path}.transport"),
                "invalid_type",
                "transport must be a string",
            ),
        }
    }

    if let Some(v) = obj.get("env") {
        validate_string_map(&format!("{path}.env"), v, issues);
    }

    if let Some(v) = obj.get("workspaceFolder")
        && !v.is_string()
    {
        push_issue(
            issues,
            &format!("{path}.workspaceFolder"),
            "invalid_type",
            "workspaceFolder must be a string",
        );
    }

    for pos_int_key in ["startupTimeout", "shutdownTimeout"] {
        if let Some(v) = obj.get(pos_int_key) {
            match v.as_i64() {
                Some(i) if i > 0 => {}
                Some(_) => push_issue(
                    issues,
                    &format!("{path}.{pos_int_key}"),
                    "too_small",
                    &format!("{pos_int_key} must be > 0"),
                ),
                None => push_issue(
                    issues,
                    &format!("{path}.{pos_int_key}"),
                    "invalid_type",
                    &format!("{pos_int_key} must be an integer"),
                ),
            }
        }
    }

    if let Some(v) = obj.get("restartOnCrash")
        && !v.is_boolean()
    {
        push_issue(
            issues,
            &format!("{path}.restartOnCrash"),
            "invalid_type",
            "restartOnCrash must be a boolean",
        );
    }

    if let Some(v) = obj.get("maxRestarts") {
        match v.as_i64() {
            Some(i) if i >= 0 => {}
            Some(_) => push_issue(
                issues,
                &format!("{path}.maxRestarts"),
                "too_small",
                "maxRestarts must be >= 0",
            ),
            None => push_issue(
                issues,
                &format!("{path}.maxRestarts"),
                "invalid_type",
                "maxRestarts must be an integer",
            ),
        }
    }
}
