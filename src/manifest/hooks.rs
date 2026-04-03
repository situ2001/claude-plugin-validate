use super::common::{is_rel_json_path, obj_get, push_issue};
use crate::ValidationIssue;
use serde_json::{Map, Value};

pub(super) fn validate_hooks_field(root: &Map<String, Value>, issues: &mut Vec<ValidationIssue>) {
    let Some(hooks) = obj_get(root, "hooks") else {
        return;
    };

    validate_hooks_value("hooks", hooks, issues);
}

fn is_valid_hook_event(event: &str) -> bool {
    matches!(
        event,
        "PreToolUse"
            | "PostToolUse"
            | "PostToolUseFailure"
            | "Notification"
            | "UserPromptSubmit"
            | "SessionStart"
            | "SessionEnd"
            | "Stop"
            | "StopFailure"
            | "SubagentStart"
            | "SubagentStop"
            | "PreCompact"
            | "PostCompact"
            | "PermissionRequest"
            | "PermissionDenied"
            | "Setup"
            | "TeammateIdle"
            | "TaskCreated"
            | "TaskCompleted"
            | "Elicitation"
            | "ElicitationResult"
            | "ConfigChange"
            | "WorktreeCreate"
            | "WorktreeRemove"
            | "InstructionsLoaded"
            | "CwdChanged"
            | "FileChanged"
    )
}

fn validate_hooks_value(path: &str, hooks: &Value, issues: &mut Vec<ValidationIssue>) {
    if let Some(s) = hooks.as_str() {
        if !is_rel_json_path(s) {
            push_issue(
                issues,
                path,
                "invalid_string",
                "hooks path must start with ./ and end with .json",
            );
        }
        return;
    }

    if let Some(arr) = hooks.as_array() {
        for (idx, item) in arr.iter().enumerate() {
            validate_hooks_value(&format!("{path}.{idx}"), item, issues);
        }
        return;
    }

    let Some(map) = hooks.as_object() else {
        push_issue(
            issues,
            path,
            "invalid_type",
            "hooks must be an object, path, or array",
        );
        return;
    };

    for (event, matchers) in map {
        if !is_valid_hook_event(event) {
            push_issue(
                issues,
                &format!("{path}.{event}"),
                "invalid_enum_value",
                "Unknown hook event",
            );
            continue;
        }

        let Some(matchers_arr) = matchers.as_array() else {
            push_issue(
                issues,
                &format!("{path}.{event}"),
                "invalid_type",
                "Hook event value must be an array",
            );
            continue;
        };

        for (m_idx, matcher) in matchers_arr.iter().enumerate() {
            let Some(m_obj) = matcher.as_object() else {
                push_issue(
                    issues,
                    &format!("{path}.{event}.{m_idx}"),
                    "invalid_type",
                    "Hook matcher must be an object",
                );
                continue;
            };

            if let Some(mv) = m_obj.get("matcher")
                && !mv.is_string()
            {
                push_issue(
                    issues,
                    &format!("{path}.{event}.{m_idx}.matcher"),
                    "invalid_type",
                    "matcher must be a string",
                );
            }

            let Some(hook_list) = m_obj.get("hooks").and_then(Value::as_array) else {
                push_issue(
                    issues,
                    &format!("{path}.{event}.{m_idx}.hooks"),
                    "missing_required",
                    "hooks must be an array",
                );
                continue;
            };

            for (h_idx, hook_cmd) in hook_list.iter().enumerate() {
                validate_hook_command(
                    &format!("{path}.{event}.{m_idx}.hooks.{h_idx}"),
                    hook_cmd,
                    issues,
                );
            }
        }
    }
}

fn validate_hook_command(path: &str, value: &Value, issues: &mut Vec<ValidationIssue>) {
    let Some(obj) = value.as_object() else {
        push_issue(
            issues,
            path,
            "invalid_type",
            "Hook command must be an object",
        );
        return;
    };

    let Some(kind) = obj.get("type").and_then(Value::as_str) else {
        push_issue(
            issues,
            &format!("{path}.type"),
            "missing_required",
            "Hook command.type is required",
        );
        return;
    };

    if let Some(ifv) = obj.get("if")
        && !ifv.is_string()
    {
        push_issue(
            issues,
            &format!("{path}.if"),
            "invalid_type",
            "if must be a string",
        );
    }

    if let Some(tv) = obj.get("timeout") {
        if let Some(num) = tv.as_f64() {
            if num <= 0.0 {
                push_issue(
                    issues,
                    &format!("{path}.timeout"),
                    "too_small",
                    "timeout must be > 0",
                );
            }
        } else {
            push_issue(
                issues,
                &format!("{path}.timeout"),
                "invalid_type",
                "timeout must be a number",
            );
        }
    }

    if let Some(msg) = obj.get("statusMessage")
        && !msg.is_string()
    {
        push_issue(
            issues,
            &format!("{path}.statusMessage"),
            "invalid_type",
            "statusMessage must be a string",
        );
    }
    for bool_key in ["once", "async", "asyncRewake"] {
        if let Some(v) = obj.get(bool_key)
            && !v.is_boolean()
        {
            push_issue(
                issues,
                &format!("{path}.{bool_key}"),
                "invalid_type",
                &format!("{bool_key} must be a boolean"),
            );
        }
    }

    match kind {
        "command" => {
            if obj.get("command").and_then(Value::as_str).is_none() {
                push_issue(
                    issues,
                    &format!("{path}.command"),
                    "missing_required",
                    "command hook requires command (string)",
                );
            }
            if let Some(shell) = obj.get("shell") {
                match shell.as_str() {
                    Some("bash") | Some("powershell") => {}
                    Some(_) => push_issue(
                        issues,
                        &format!("{path}.shell"),
                        "invalid_enum_value",
                        "shell must be bash or powershell",
                    ),
                    None => push_issue(
                        issues,
                        &format!("{path}.shell"),
                        "invalid_type",
                        "shell must be a string",
                    ),
                }
            }
        }
        "prompt" | "agent" => {
            if obj.get("prompt").and_then(Value::as_str).is_none() {
                push_issue(
                    issues,
                    &format!("{path}.prompt"),
                    "missing_required",
                    "prompt/agent hook requires prompt (string)",
                );
            }
            if let Some(model) = obj.get("model")
                && !model.is_string()
            {
                push_issue(
                    issues,
                    &format!("{path}.model"),
                    "invalid_type",
                    "model must be a string",
                );
            }
        }
        "http" => {
            match obj.get("url").and_then(Value::as_str) {
                Some(url) if url::Url::parse(url).is_ok() => {}
                Some(_) => push_issue(
                    issues,
                    &format!("{path}.url"),
                    "invalid_string",
                    "http hook url must be a valid URL",
                ),
                None => push_issue(
                    issues,
                    &format!("{path}.url"),
                    "missing_required",
                    "http hook requires url",
                ),
            }

            if let Some(headers) = obj.get("headers") {
                let Some(hm) = headers.as_object() else {
                    push_issue(
                        issues,
                        &format!("{path}.headers"),
                        "invalid_type",
                        "headers must be a string map",
                    );
                    return;
                };
                for (k, v) in hm {
                    if !v.is_string() {
                        push_issue(
                            issues,
                            &format!("{path}.headers.{k}"),
                            "invalid_type",
                            "header values must be strings",
                        );
                    }
                }
            }

            if let Some(allowed) = obj.get("allowedEnvVars") {
                let Some(arr) = allowed.as_array() else {
                    push_issue(
                        issues,
                        &format!("{path}.allowedEnvVars"),
                        "invalid_type",
                        "allowedEnvVars must be an array of strings",
                    );
                    return;
                };
                for (i, item) in arr.iter().enumerate() {
                    if !item.is_string() {
                        push_issue(
                            issues,
                            &format!("{path}.allowedEnvVars.{i}"),
                            "invalid_type",
                            "allowedEnvVars item must be a string",
                        );
                    }
                }
            }
        }
        _ => push_issue(
            issues,
            &format!("{path}.type"),
            "invalid_enum_value",
            "Hook command.type must be one of: command|prompt|agent|http",
        ),
    }
}
