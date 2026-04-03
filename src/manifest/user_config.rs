use super::common::{obj_get, push_issue};
use crate::ValidationIssue;
use serde_json::{Map, Value};

pub(super) fn validate_user_config(
    root: &Map<String, Value>,
    issues: &mut Vec<ValidationIssue>,
    user_config_key_re: &regex::Regex,
) {
    let Some(user_cfg) = obj_get(root, "userConfig") else {
        return;
    };

    let Some(map) = user_cfg.as_object() else {
        push_issue(
            issues,
            "userConfig",
            "invalid_type",
            "userConfig must be an object",
        );
        return;
    };

    for (key, option) in map {
        if !user_config_key_re.is_match(key) {
            push_issue(
                issues,
                &format!("userConfig.{key}"),
                "invalid_string",
                "userConfig key must be a valid identifier",
            );
        }

        validate_user_config_option(&format!("userConfig.{key}"), option, issues);
    }
}

pub(super) fn validate_user_config_option(path: &str, option: &Value, issues: &mut Vec<ValidationIssue>) {
    let Some(obj) = option.as_object() else {
        push_issue(
            issues,
            path,
            "invalid_type",
            "userConfig option must be an object",
        );
        return;
    };

    match obj.get("type").and_then(Value::as_str) {
        Some("string") | Some("number") | Some("boolean") | Some("directory") | Some("file") => {}
        Some(_) => push_issue(
            issues,
            &format!("{path}.type"),
            "invalid_enum_value",
            "type must be one of string|number|boolean|directory|file",
        ),
        None => push_issue(
            issues,
            &format!("{path}.type"),
            "missing_required",
            "type is required",
        ),
    }

    for key in ["title", "description"] {
        match obj.get(key).and_then(Value::as_str) {
            Some(s) if !s.is_empty() => {}
            Some(_) => push_issue(
                issues,
                &format!("{path}.{key}"),
                "too_small",
                &format!("{key} must be non-empty"),
            ),
            None => push_issue(
                issues,
                &format!("{path}.{key}"),
                "missing_required",
                &format!("{key} is required"),
            ),
        }
    }

    if let Some(default) = obj.get("default")
        && !is_default_union_value(default)
    {
        push_issue(
            issues,
            &format!("{path}.default"),
            "invalid_type",
            "default must be string|number|boolean|string[]",
        );
    }

    if let Some(multiple) = obj.get("multiple")
        && !multiple.is_boolean()
    {
        push_issue(
            issues,
            &format!("{path}.multiple"),
            "invalid_type",
            "multiple must be a boolean",
        );
    }
    if let Some(sensitive) = obj.get("sensitive")
        && !sensitive.is_boolean()
    {
        push_issue(
            issues,
            &format!("{path}.sensitive"),
            "invalid_type",
            "sensitive must be a boolean",
        );
    }
    if let Some(required) = obj.get("required")
        && !required.is_boolean()
    {
        push_issue(
            issues,
            &format!("{path}.required"),
            "invalid_type",
            "required must be a boolean",
        );
    }
    for key in ["min", "max"] {
        if let Some(v) = obj.get(key)
            && !v.is_number()
        {
            push_issue(
                issues,
                &format!("{path}.{key}"),
                "invalid_type",
                &format!("{key} must be a number"),
            );
        }
    }

    let allowed = [
        "type",
        "title",
        "description",
        "required",
        "default",
        "multiple",
        "sensitive",
        "min",
        "max",
    ];
    for key in obj.keys() {
        if !allowed.contains(&key.as_str()) {
            push_issue(
                issues,
                path,
                "unrecognized_keys",
                &format!("Unrecognized key in userConfig option: {key}"),
            );
        }
    }
}

fn is_default_union_value(value: &Value) -> bool {
    if value.is_string() || value.is_number() || value.is_boolean() {
        return true;
    }
    if let Some(arr) = value.as_array() {
        return arr.iter().all(Value::is_string);
    }
    false
}
