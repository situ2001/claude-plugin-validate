use super::common::{is_rel_path, obj_get, push_issue};
use crate::ValidationIssue;
use serde_json::{Map, Value};

pub(super) fn validate_commands_field(
    root: &Map<String, Value>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(commands) = obj_get(root, "commands") else {
        return;
    };

    validate_commands_value("commands", commands, issues);
}

fn validate_commands_value(path: &str, value: &Value, issues: &mut Vec<ValidationIssue>) {
    if let Some(s) = value.as_str() {
        if !is_rel_path(s) {
            push_issue(
                issues,
                path,
                "invalid_string",
                "Path must be relative and start with ./",
            );
            return;
        }
        return;
    }

    if let Some(arr) = value.as_array() {
        for (idx, item) in arr.iter().enumerate() {
            validate_commands_value(&format!("{path}.{idx}"), item, issues);
        }
        return;
    }

    let Some(map) = value.as_object() else {
        push_issue(
            issues,
            path,
            "invalid_type",
            "commands must be string | array | object",
        );
        return;
    };

    for (name, metadata) in map {
        let Some(meta) = metadata.as_object() else {
            push_issue(
                issues,
                &format!("{path}.{name}"),
                "invalid_type",
                "command metadata must be an object",
            );
            continue;
        };

        let has_source = meta.get("source").is_some();
        let has_content = meta.get("content").is_some();
        if has_source == has_content {
            push_issue(
                issues,
                &format!("{path}.{name}"),
                "invalid_union",
                "Command metadata requires either source or content, but not both",
            );
        }

        if let Some(source) = meta.get("source").and_then(Value::as_str)
            && !is_rel_path(source)
        {
            push_issue(
                issues,
                &format!("{path}.{name}.source"),
                "invalid_string",
                "Path must be relative and start with ./",
            );
        }

        if let Some(content) = meta.get("content")
            && !content.is_string()
        {
            push_issue(
                issues,
                &format!("{path}.{name}.content"),
                "invalid_type",
                "content must be a string",
            );
        }

        for key in ["description", "argumentHint", "model"] {
            if let Some(v) = meta.get(key)
                && !v.is_string()
            {
                push_issue(
                    issues,
                    &format!("{path}.{name}.{key}"),
                    "invalid_type",
                    &format!("{key} must be a string"),
                );
            }
        }

        if let Some(allowed_tools) = meta.get("allowedTools") {
            let Some(arr) = allowed_tools.as_array() else {
                push_issue(
                    issues,
                    &format!("{path}.{name}.allowedTools"),
                    "invalid_type",
                    "allowedTools must be an array of strings",
                );
                continue;
            };
            for (i, item) in arr.iter().enumerate() {
                if !item.is_string() {
                    push_issue(
                        issues,
                        &format!("{path}.{name}.allowedTools.{i}"),
                        "invalid_type",
                        "allowedTools item must be a string",
                    );
                }
            }
        }
    }
}
