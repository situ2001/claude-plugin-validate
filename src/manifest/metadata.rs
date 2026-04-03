use super::common::{obj_get, push_issue};
use crate::ValidationIssue;
use serde_json::{Map, Value};

pub(super) fn validate_required_name(
    root: &Map<String, Value>,
    issues: &mut Vec<ValidationIssue>,
    plugin_name_re: &regex::Regex,
) {
    let Some(name) = obj_get(root, "name") else {
        push_issue(
            issues,
            "name",
            "missing_required",
            "Missing required field: name",
        );
        return;
    };

    let Some(name_str) = name.as_str() else {
        push_issue(issues, "name", "invalid_type", "name must be a string");
        return;
    };

    if name_str.is_empty() {
        push_issue(issues, "name", "too_small", "Plugin name cannot be empty");
        return;
    }

    if !plugin_name_re.is_match(name_str) {
        push_issue(
            issues,
            "name",
            "invalid_string",
            "Plugin name contains invalid characters",
        );
    }
}

pub(super) fn validate_optional_metadata(root: &Map<String, Value>, issues: &mut Vec<ValidationIssue>) {
    for key in ["version", "description", "repository", "license"] {
        if let Some(v) = obj_get(root, key)
            && !v.is_string()
        {
            push_issue(
                issues,
                key,
                "invalid_type",
                &format!("{key} must be a string"),
            );
        }
    }

    if let Some(v) = obj_get(root, "homepage")
        && let Some(s) = v.as_str()
    {
        if url::Url::parse(s).is_err() {
            push_issue(
                issues,
                "homepage",
                "invalid_string",
                "homepage must be a valid URL",
            );
        }
    } else if obj_get(root, "homepage").is_some() {
        push_issue(
            issues,
            "homepage",
            "invalid_type",
            "homepage must be a string URL",
        );
    }

    if let Some(author) = obj_get(root, "author") {
        let Some(obj) = author.as_object() else {
            push_issue(issues, "author", "invalid_type", "author must be an object");
            return;
        };

        match obj.get("name") {
            Some(v) if v.as_str().map(|s| !s.is_empty()).unwrap_or(false) => {}
            Some(_) => push_issue(
                issues,
                "author.name",
                "invalid_type",
                "author.name must be a non-empty string",
            ),
            None => push_issue(
                issues,
                "author.name",
                "missing_required",
                "author.name is required when author is provided",
            ),
        }

        for k in ["email", "url"] {
            if let Some(v) = obj.get(k)
                && !v.is_string()
            {
                push_issue(
                    issues,
                    &format!("author.{k}"),
                    "invalid_type",
                    &format!("author.{k} must be a string"),
                );
            }
        }
    }

    if let Some(keywords) = obj_get(root, "keywords") {
        let Some(arr) = keywords.as_array() else {
            push_issue(
                issues,
                "keywords",
                "invalid_type",
                "keywords must be an array of strings",
            );
            return;
        };
        for (idx, item) in arr.iter().enumerate() {
            if !item.is_string() {
                push_issue(
                    issues,
                    &format!("keywords.{idx}"),
                    "invalid_type",
                    "keyword must be a string",
                );
            }
        }
    }
}
