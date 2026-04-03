use super::common::{is_rel_path, obj_get, push_issue};
use crate::ValidationIssue;
use serde_json::{Map, Value};

pub(super) fn validate_agents_skills_output_styles(
    root: &Map<String, Value>,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_path_or_path_array(root, "agents", true, issues);
    validate_path_or_path_array(root, "skills", false, issues);
    validate_path_or_path_array(root, "outputStyles", false, issues);
}

fn validate_path_or_path_array(
    root: &Map<String, Value>,
    key: &str,
    markdown_only: bool,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(value) = obj_get(root, key) else {
        return;
    };

    let validate_one = |path: &str| -> Option<&'static str> {
        if !is_rel_path(path) {
            return Some("Path must be relative and start with ./");
        }
        if markdown_only && !path.ends_with(".md") {
            return Some("Path must point to a .md file");
        }
        None
    };

    if let Some(s) = value.as_str() {
        if let Some(msg) = validate_one(s) {
            push_issue(issues, key, "invalid_string", msg);
        }
        return;
    }

    if let Some(arr) = value.as_array() {
        for (idx, item) in arr.iter().enumerate() {
            match item.as_str() {
                Some(s) => {
                    if let Some(msg) = validate_one(s) {
                        push_issue(issues, &format!("{key}.{idx}"), "invalid_string", msg);
                    }
                }
                None => push_issue(
                    issues,
                    &format!("{key}.{idx}"),
                    "invalid_type",
                    "Item must be a string path",
                ),
            }
        }
        return;
    }

    push_issue(
        issues,
        key,
        "invalid_type",
        "Value must be a string path or array of string paths",
    );
}
