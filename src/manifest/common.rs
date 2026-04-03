use crate::ValidationIssue;
use serde_json::{Map, Value};

pub(crate) fn issue(path: &str, code: &str, message: &str) -> ValidationIssue {
    ValidationIssue {
        path: path.to_string(),
        code: code.to_string(),
        message: message.to_string(),
    }
}

pub(crate) fn push_issue(
    issues: &mut Vec<ValidationIssue>,
    path: &str,
    code: &str,
    message: &str,
) {
    issues.push(issue(
        if path.is_empty() { "<root>" } else { path },
        code,
        message,
    ));
}

pub(crate) fn obj_get<'a>(root: &'a Map<String, Value>, key: &str) -> Option<&'a Value> {
    root.get(key)
}

pub(crate) fn is_rel_path(s: &str) -> bool {
    s.starts_with("./")
}

pub(crate) fn is_rel_json_path(s: &str) -> bool {
    is_rel_path(s) && s.ends_with(".json")
}

pub(crate) fn validate_string_map(path: &str, value: &Value, issues: &mut Vec<ValidationIssue>) {
    let Some(map) = value.as_object() else {
        push_issue(
            issues,
            path,
            "invalid_type",
            "Expected object map of strings",
        );
        return;
    };

    for (k, v) in map {
        if !v.is_string() {
            push_issue(
                issues,
                &format!("{path}.{k}"),
                "invalid_type",
                "Map value must be a string",
            );
        }
    }
}
