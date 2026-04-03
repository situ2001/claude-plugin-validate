use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

use crate::ValidationResult;

mod channels_settings;
mod commands;
mod common;
mod dependencies;
mod hooks;
mod lsp_servers;
mod mcp_servers;
mod metadata;
mod path_fields;
mod user_config;

static PLUGIN_NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z0-9][-a-z0-9._]*$").expect("valid plugin name regex"));
static DEP_STR_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z0-9][-a-z0-9._]*(?:@[a-z0-9][-a-z0-9._]*)?(?:@\^[^@]*)?$")
        .expect("valid dependency regex")
});
static DEP_NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z0-9][-a-z0-9._]*$").expect("valid dep name regex"));
static USER_CONFIG_KEY_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z_]\w*$").expect("valid user config key regex"));

pub fn validate_plugin_manifest(input: Value) -> ValidationResult {
    let mut issues = Vec::new();

    let Some(root) = input.as_object() else {
        common::push_issue(
            &mut issues,
            "",
            "invalid_type",
            "Manifest root must be a JSON object",
        );
        return ValidationResult::Failure { issues };
    };

    metadata::validate_required_name(root, &mut issues, &PLUGIN_NAME_RE);
    metadata::validate_optional_metadata(root, &mut issues);
    dependencies::validate_dependencies(root, &mut issues, &DEP_STR_RE, &DEP_NAME_RE);
    hooks::validate_hooks_field(root, &mut issues);
    commands::validate_commands_field(root, &mut issues);
    path_fields::validate_agents_skills_output_styles(root, &mut issues);
    mcp_servers::validate_mcp_servers_field(root, &mut issues);
    lsp_servers::validate_lsp_servers_field(root, &mut issues);
    user_config::validate_user_config(root, &mut issues, &USER_CONFIG_KEY_RE);
    channels_settings::validate_channels(root, &mut issues);
    channels_settings::validate_settings(root, &mut issues);

    if issues.is_empty() {
        ValidationResult::Success { data: input }
    } else {
        ValidationResult::Failure { issues }
    }
}
