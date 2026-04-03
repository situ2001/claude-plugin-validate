use super::common::{obj_get, push_issue};
use super::user_config::validate_user_config_option;
use crate::ValidationIssue;
use serde_json::{Map, Value};

pub(super) fn validate_channels(root: &Map<String, Value>, issues: &mut Vec<ValidationIssue>) {
    let Some(channels) = obj_get(root, "channels") else {
        return;
    };

    let Some(arr) = channels.as_array() else {
        push_issue(
            issues,
            "channels",
            "invalid_type",
            "channels must be an array",
        );
        return;
    };

    for (idx, channel) in arr.iter().enumerate() {
        let Some(obj) = channel.as_object() else {
            push_issue(
                issues,
                &format!("channels.{idx}"),
                "invalid_type",
                "channel entry must be an object",
            );
            continue;
        };

        match obj.get("server").and_then(Value::as_str) {
            Some(s) if !s.is_empty() => {}
            Some(_) => push_issue(
                issues,
                &format!("channels.{idx}.server"),
                "too_small",
                "server must be a non-empty string",
            ),
            None => push_issue(
                issues,
                &format!("channels.{idx}.server"),
                "missing_required",
                "Missing required field: server",
            ),
        }

        if let Some(display_name) = obj.get("displayName")
            && !display_name.is_string()
        {
            push_issue(
                issues,
                &format!("channels.{idx}.displayName"),
                "invalid_type",
                "displayName must be a string",
            );
        }

        if let Some(user_cfg) = obj.get("userConfig") {
            let Some(user_cfg_map) = user_cfg.as_object() else {
                push_issue(
                    issues,
                    &format!("channels.{idx}.userConfig"),
                    "invalid_type",
                    "userConfig must be an object",
                );
                continue;
            };
            for (cfg_key, cfg_val) in user_cfg_map {
                validate_user_config_option(
                    &format!("channels.{idx}.userConfig.{cfg_key}"),
                    cfg_val,
                    issues,
                );
            }
        }

        for key in obj.keys() {
            if key != "server" && key != "displayName" && key != "userConfig" {
                push_issue(
                    issues,
                    &format!("channels.{idx}"),
                    "unrecognized_keys",
                    &format!("Unrecognized key: {key}"),
                );
            }
        }
    }
}

pub(super) fn validate_settings(root: &Map<String, Value>, issues: &mut Vec<ValidationIssue>) {
    if let Some(settings) = obj_get(root, "settings")
        && !settings.is_object()
    {
        push_issue(
            issues,
            "settings",
            "invalid_type",
            "settings must be an object map",
        );
    }
}
