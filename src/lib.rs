use serde_json::Value;

mod component;
mod manifest;

pub use component::validate_component_markdown;
pub use manifest::validate_plugin_manifest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub path: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Success { data: Value },
    Failure { issues: Vec<ValidationIssue> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentValidation {
    pub path: String,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_valid_minimal() {
        let input = serde_json::json!({
            "name": "demo-plugin",
            "commands": {
                "about": {
                    "source": "./commands/about.md"
                }
            }
        });

        let result = validate_plugin_manifest(input);
        match result {
            ValidationResult::Success { .. } => {}
            ValidationResult::Failure { issues } => {
                panic!("expected success, got: {issues:#?}")
            }
        }
    }

    #[test]
    fn manifest_invalid_should_fail() {
        let input = serde_json::json!({
            "name": "bad plugin",
            "commands": {
                "x": {
                    "source": "commands/no-prefix.md",
                    "content": "bad"
                }
            },
            "hooks": {
                "NotARealHookEvent": []
            }
        });

        let result = validate_plugin_manifest(input);
        match result {
            ValidationResult::Failure { issues } => {
                assert!(!issues.is_empty());
            }
            ValidationResult::Success { .. } => panic!("expected failure"),
        }
    }

    #[test]
    fn component_frontmatter_validation() {
        let markdown = "---\nname: reviewer\ndescription: checks code\nallowed-tools:\n  - Bash\nshell: bash\n---\n\n# Reviewer\n";
        let result = validate_component_markdown("agents/reviewer.md", markdown, "agent");
        assert!(result.errors.is_empty());
    }

    #[test]
    fn component_invalid_frontmatter_validation() {
        let markdown =
            "---\nname: 1\ndescription:\n  nested: yes\nshell: fish\n---\n\n# Reviewer\n";
        let result = validate_component_markdown("agents/reviewer.md", markdown, "agent");
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn fixture_manifest_valid() {
        let s =
            std::fs::read_to_string("fixtures/manifest/valid/plugin.json").expect("read fixture");
        let v: Value = serde_json::from_str(&s).expect("json");
        match validate_plugin_manifest(v) {
            ValidationResult::Success { .. } => {}
            ValidationResult::Failure { issues } => {
                panic!("expected fixture valid to pass, got {issues:#?}")
            }
        }
    }

    #[test]
    fn fixture_manifest_invalid() {
        let s =
            std::fs::read_to_string("fixtures/manifest/invalid/plugin.json").expect("read fixture");
        let v: Value = serde_json::from_str(&s).expect("json");
        match validate_plugin_manifest(v) {
            ValidationResult::Failure { issues } => assert!(!issues.is_empty()),
            ValidationResult::Success { .. } => panic!("expected fixture invalid to fail"),
        }
    }

    #[test]
    fn fixture_component_valid() {
        let s = std::fs::read_to_string("fixtures/components/valid/agent.md").expect("read md");
        let r = validate_component_markdown("fixtures/components/valid/agent.md", &s, "agent");
        assert!(r.errors.is_empty());
    }

    #[test]
    fn fixture_component_invalid() {
        let s = std::fs::read_to_string("fixtures/components/invalid/agent.md").expect("read md");
        let r = validate_component_markdown("fixtures/components/invalid/agent.md", &s, "agent");
        assert!(!r.errors.is_empty());
    }
}
