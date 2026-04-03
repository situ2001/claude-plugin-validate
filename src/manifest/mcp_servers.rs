use super::common::{is_rel_json_path, is_rel_path, obj_get, push_issue, validate_string_map};
use crate::ValidationIssue;
use serde_json::{Map, Value};

pub(super) fn validate_mcp_servers_field(
    root: &Map<String, Value>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(value) = obj_get(root, "mcpServers") else {
        return;
    };

    validate_mcp_servers_value("mcpServers", value, issues);
}

fn validate_mcp_servers_value(path: &str, value: &Value, issues: &mut Vec<ValidationIssue>) {
    if let Some(s) = value.as_str() {
        if is_rel_json_path(s) {
            return;
        }
        if (s.ends_with(".mcpb") || s.ends_with(".dxt"))
            && (is_rel_path(s) || url::Url::parse(s).is_ok())
        {
            return;
        }
        push_issue(
            issues,
            path,
            "invalid_string",
            "Expected .json relative path, or .mcpb/.dxt relative path/URL",
        );
        return;
    }

    if let Some(arr) = value.as_array() {
        for (idx, item) in arr.iter().enumerate() {
            validate_mcp_servers_value(&format!("{path}.{idx}"), item, issues);
        }
        return;
    }

    let Some(map) = value.as_object() else {
        push_issue(
            issues,
            path,
            "invalid_type",
            "mcpServers must be path | map | array",
        );
        return;
    };

    for (server_name, cfg) in map {
        validate_mcp_server_config(&format!("{path}.{server_name}"), cfg, issues);
    }
}

fn validate_mcp_server_config(path: &str, cfg: &Value, issues: &mut Vec<ValidationIssue>) {
    let Some(obj) = cfg.as_object() else {
        push_issue(
            issues,
            path,
            "invalid_type",
            "MCP server config must be an object",
        );
        return;
    };

    let t = obj.get("type").and_then(Value::as_str).unwrap_or("stdio");

    match t {
        "stdio" => {
            if obj.get("command").and_then(Value::as_str).is_none() {
                push_issue(
                    issues,
                    &format!("{path}.command"),
                    "missing_required",
                    "stdio config requires command",
                );
            }
            if let Some(args) = obj.get("args") {
                let Some(arr) = args.as_array() else {
                    push_issue(
                        issues,
                        &format!("{path}.args"),
                        "invalid_type",
                        "args must be an array of strings",
                    );
                    return;
                };
                for (idx, item) in arr.iter().enumerate() {
                    if !item.is_string() {
                        push_issue(
                            issues,
                            &format!("{path}.args.{idx}"),
                            "invalid_type",
                            "arg must be a string",
                        );
                    }
                }
            }
            if let Some(env) = obj.get("env") {
                validate_string_map(&format!("{path}.env"), env, issues);
            }
        }
        "sse" | "http" => {
            if obj.get("url").and_then(Value::as_str).is_none() {
                push_issue(
                    issues,
                    &format!("{path}.url"),
                    "missing_required",
                    &format!("{t} config requires url"),
                );
            }
            if let Some(headers) = obj.get("headers") {
                validate_string_map(&format!("{path}.headers"), headers, issues);
            }
            if let Some(headers_helper) = obj.get("headersHelper")
                && !headers_helper.is_string()
            {
                push_issue(
                    issues,
                    &format!("{path}.headersHelper"),
                    "invalid_type",
                    "headersHelper must be a string",
                );
            }
            if let Some(oauth) = obj.get("oauth") {
                validate_oauth_config(&format!("{path}.oauth"), oauth, issues);
            }
        }
        "sse-ide" => {
            if obj.get("url").and_then(Value::as_str).is_none() {
                push_issue(
                    issues,
                    &format!("{path}.url"),
                    "missing_required",
                    "sse-ide config requires url",
                );
            }
            if obj.get("ideName").and_then(Value::as_str).is_none() {
                push_issue(
                    issues,
                    &format!("{path}.ideName"),
                    "missing_required",
                    "sse-ide config requires ideName",
                );
            }
            if let Some(v) = obj.get("ideRunningInWindows")
                && !v.is_boolean()
            {
                push_issue(
                    issues,
                    &format!("{path}.ideRunningInWindows"),
                    "invalid_type",
                    "ideRunningInWindows must be a boolean",
                );
            }
        }
        "ws-ide" => {
            if obj.get("url").and_then(Value::as_str).is_none() {
                push_issue(
                    issues,
                    &format!("{path}.url"),
                    "missing_required",
                    "ws-ide config requires url",
                );
            }
            if obj.get("ideName").and_then(Value::as_str).is_none() {
                push_issue(
                    issues,
                    &format!("{path}.ideName"),
                    "missing_required",
                    "ws-ide config requires ideName",
                );
            }
            if let Some(v) = obj.get("authToken")
                && !v.is_string()
            {
                push_issue(
                    issues,
                    &format!("{path}.authToken"),
                    "invalid_type",
                    "authToken must be a string",
                );
            }
            if let Some(v) = obj.get("ideRunningInWindows")
                && !v.is_boolean()
            {
                push_issue(
                    issues,
                    &format!("{path}.ideRunningInWindows"),
                    "invalid_type",
                    "ideRunningInWindows must be a boolean",
                );
            }
        }
        "ws" => {
            if obj.get("url").and_then(Value::as_str).is_none() {
                push_issue(
                    issues,
                    &format!("{path}.url"),
                    "missing_required",
                    "ws config requires url",
                );
            }
            if let Some(headers) = obj.get("headers") {
                validate_string_map(&format!("{path}.headers"), headers, issues);
            }
            if let Some(headers_helper) = obj.get("headersHelper")
                && !headers_helper.is_string()
            {
                push_issue(
                    issues,
                    &format!("{path}.headersHelper"),
                    "invalid_type",
                    "headersHelper must be a string",
                );
            }
        }
        "sdk" => {
            if obj.get("name").and_then(Value::as_str).is_none() {
                push_issue(
                    issues,
                    &format!("{path}.name"),
                    "missing_required",
                    "sdk config requires name",
                );
            }
        }
        "claudeai-proxy" => {
            if obj.get("url").and_then(Value::as_str).is_none() {
                push_issue(
                    issues,
                    &format!("{path}.url"),
                    "missing_required",
                    "claudeai-proxy config requires url",
                );
            }
            if obj.get("id").and_then(Value::as_str).is_none() {
                push_issue(
                    issues,
                    &format!("{path}.id"),
                    "missing_required",
                    "claudeai-proxy config requires id",
                );
            }
        }
        _ => push_issue(
            issues,
            &format!("{path}.type"),
            "invalid_enum_value",
            "Unsupported MCP server type",
        ),
    }
}

fn validate_oauth_config(path: &str, value: &Value, issues: &mut Vec<ValidationIssue>) {
    let Some(obj) = value.as_object() else {
        push_issue(issues, path, "invalid_type", "oauth must be an object");
        return;
    };

    if let Some(client_id) = obj.get("clientId")
        && !client_id.is_string()
    {
        push_issue(
            issues,
            &format!("{path}.clientId"),
            "invalid_type",
            "clientId must be a string",
        );
    }

    if let Some(cb) = obj.get("callbackPort") {
        match cb.as_i64() {
            Some(v) if v > 0 => {}
            Some(_) => push_issue(
                issues,
                &format!("{path}.callbackPort"),
                "too_small",
                "callbackPort must be > 0",
            ),
            None => push_issue(
                issues,
                &format!("{path}.callbackPort"),
                "invalid_type",
                "callbackPort must be an integer",
            ),
        }
    }

    if let Some(auth_url) = obj.get("authServerMetadataUrl") {
        let Some(url) = auth_url.as_str() else {
            push_issue(
                issues,
                &format!("{path}.authServerMetadataUrl"),
                "invalid_type",
                "authServerMetadataUrl must be a string",
            );
            return;
        };

        if !url.starts_with("https://") || url::Url::parse(url).is_err() {
            push_issue(
                issues,
                &format!("{path}.authServerMetadataUrl"),
                "invalid_string",
                "authServerMetadataUrl must use https:// and be a valid URL",
            );
        }
    }

    if let Some(xaa) = obj.get("xaa")
        && !xaa.is_boolean()
    {
        push_issue(
            issues,
            &format!("{path}.xaa"),
            "invalid_type",
            "xaa must be a boolean",
        );
    }
}
