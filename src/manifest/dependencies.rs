use super::common::{obj_get, push_issue};
use crate::ValidationIssue;
use serde_json::{Map, Value};

pub(super) fn validate_dependencies(
    root: &Map<String, Value>,
    issues: &mut Vec<ValidationIssue>,
    dep_str_re: &regex::Regex,
    dep_name_re: &regex::Regex,
) {
    let Some(deps) = obj_get(root, "dependencies") else {
        return;
    };
    let Some(arr) = deps.as_array() else {
        push_issue(
            issues,
            "dependencies",
            "invalid_type",
            "dependencies must be an array",
        );
        return;
    };

    for (idx, dep) in arr.iter().enumerate() {
        if let Some(s) = dep.as_str() {
            if !dep_str_re.is_match(s) {
                push_issue(
                    issues,
                    &format!("dependencies.{idx}"),
                    "invalid_string",
                    "Invalid dependency reference format",
                );
            }
            continue;
        }

        let Some(obj) = dep.as_object() else {
            push_issue(
                issues,
                &format!("dependencies.{idx}"),
                "invalid_type",
                "Dependency must be a string or object",
            );
            continue;
        };

        match obj.get("name").and_then(Value::as_str) {
            Some(name) if dep_name_re.is_match(name) => {}
            _ => push_issue(
                issues,
                &format!("dependencies.{idx}.name"),
                "invalid_string",
                "Invalid dependency name",
            ),
        }

        if let Some(mkt) = obj.get("marketplace")
            && let Some(mkt_s) = mkt.as_str()
        {
            if !dep_name_re.is_match(mkt_s) {
                push_issue(
                    issues,
                    &format!("dependencies.{idx}.marketplace"),
                    "invalid_string",
                    "Invalid marketplace name",
                );
            }
        } else if obj.get("marketplace").is_some() {
            push_issue(
                issues,
                &format!("dependencies.{idx}.marketplace"),
                "invalid_type",
                "marketplace must be a string",
            );
        }
    }
}
