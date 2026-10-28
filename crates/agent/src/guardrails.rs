use anyhow::{Context as _, Result};
use globset::{Glob, GlobMatcher, GlobSet, GlobSetBuilder};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, LazyLock};
use crate::ToolPermissionDecision;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PathRule {
    pub pattern: String,
    pub action: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct GuardrailConfig {
    #[serde(default)]
    pub workspace_id: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub deny_paths: Vec<String>,
    #[serde(default)]
    pub path_rules: Vec<PathRule>,
    #[serde(default)]
    pub protected_branches: Vec<String>,
    #[serde(default)]
    pub allowed_commands: Vec<String>,
    #[serde(default)]
    pub rag_service_url: Option<String>,
}

pub struct CompiledGuardrails {
    deny_matcher: GlobSet,
    path_rules: Vec<(GlobMatcher, PathRule)>,
    allowed_commands: Vec<String>,
    raw_config: GuardrailConfig,
}

static ACTIVE_GUARDRAILS: LazyLock<RwLock<Option<Arc<CompiledGuardrails>>>> =
    LazyLock::new(|| RwLock::new(None));

impl CompiledGuardrails {
    pub fn compile(config: GuardrailConfig) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();
        for pattern in &config.deny_paths {
            let glob = Glob::new(pattern).with_context(|| format!("Invalid deny_path glob: {pattern}"))?;
            builder.add(glob);
        }
        let deny_matcher = builder
            .build()
            .with_context(|| "Failed to compile deny_paths globset")?;

        let mut path_rules = Vec::new();
        for rule in &config.path_rules {
            let glob = Glob::new(&rule.pattern)
                .with_context(|| format!("Invalid path_rule glob: {}", rule.pattern))?;
            path_rules.push((glob.compile_matcher(), rule.clone()));
        }

        Ok(Self {
            deny_matcher,
            path_rules,
            allowed_commands: config.allowed_commands.clone(),
            raw_config: config,
        })
    }

    pub fn raw_config(&self) -> &GuardrailConfig {
        &self.raw_config
    }

    pub fn check_path(&self, raw_path: &str) -> Option<ToolPermissionDecision> {
        let normalized = raw_path
            .trim_start_matches('/')
            .trim_start_matches("./");

        for (matcher, rule) in &self.path_rules {
            if matcher.is_match(normalized) || matcher.is_match(raw_path) {
                if rule.action.eq_ignore_ascii_case("block") || rule.action.eq_ignore_ascii_case("deny") {
                    return Some(ToolPermissionDecision::Deny(format!(
                        "Guardrail blocked: {}",
                        rule.reason
                    )));
                }
            }
        }

        if self.deny_matcher.is_match(normalized) || self.deny_matcher.is_match(raw_path) {
            return Some(ToolPermissionDecision::Deny(format!(
                "Guardrail blocked: Direct access to path '{raw_path}' is forbidden by project guardrail rules."
            )));
        }

        None
    }

    pub fn check_terminal_command(&self, command: &str) -> Option<ToolPermissionDecision> {
        if self.allowed_commands.is_empty() {
            return None;
        }

        let trimmed_command = command.trim();
        for allowed in &self.allowed_commands {
            if trimmed_command.starts_with(allowed) {
                return None;
            }
        }

        Some(ToolPermissionDecision::Deny(format!(
            "Guardrail blocked: Command '{command}' is not in the allowed commands whitelist: {:?}",
            self.allowed_commands
        )))
    }
}

pub fn set_active_guardrails(config: GuardrailConfig) -> Result<()> {
    let compiled = CompiledGuardrails::compile(config)?;
    let mut lock = ACTIVE_GUARDRAILS.write();
    *lock = Some(Arc::new(compiled));
    Ok(())
}

pub fn clear_active_guardrails() {
    let mut lock = ACTIVE_GUARDRAILS.write();
    *lock = None;
}

pub fn get_active_guardrails() -> Option<Arc<CompiledGuardrails>> {
    ACTIVE_GUARDRAILS.read().clone()
}

pub fn check_guardrail_permissions(
    tool_name: &str,
    inputs: &[String],
) -> Option<ToolPermissionDecision> {
    let guardrails = get_active_guardrails()?;

    const FILE_MODIFICATION_TOOLS: &[&str] = &[
        "write_file",
        "edit_file",
        "delete_path",
        "create_directory",
        "move_path",
        "copy_path",
    ];

    if FILE_MODIFICATION_TOOLS.contains(&tool_name) {
        for input in inputs {
            if let Some(decision) = guardrails.check_path(input) {
                return Some(decision);
            }
        }
    } else if tool_name == "terminal" {
        for input in inputs {
            if let Some(decision) = guardrails.check_terminal_command(input) {
                return Some(decision);
            }
        }
    }

    None
}

pub fn load_guardrails_from_str(content: &str) -> Result<GuardrailConfig> {
    // Attempt parsing as JSON first, falling back to basic key-value extraction if needed
    if let Ok(config) = serde_json::from_str::<GuardrailConfig>(content) {
        return Ok(config);
    }

    #[derive(Deserialize)]
    struct Wrapper {
        guardrails: GuardrailConfig,
    }
    if let Ok(wrapper) = serde_json::from_str::<Wrapper>(content) {
        return Ok(wrapper.guardrails);
    }

    anyhow::bail!("Failed to parse guardrail configuration")
}

pub fn load_guardrails_from_file(path: &Path) -> Result<GuardrailConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read guardrails file at {}", path.display()))?;
    load_guardrails_from_str(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guardrails_deny_path() {
        let config = GuardrailConfig {
            workspace_id: "test".to_string(),
            version: "1.0".to_string(),
            deny_paths: vec!["common-grpc/**".to_string(), "proto/gen/**".to_string()],
            path_rules: vec![PathRule {
                pattern: "common-grpc/**".to_string(),
                action: "block".to_string(),
                reason: "Direct edits to common-grpc forbidden. Update proto schemas in services instead.".to_string(),
            }],
            protected_branches: vec!["main".to_string()],
            allowed_commands: vec![],
            rag_service_url: None,
        };

        let compiled = CompiledGuardrails::compile(config).unwrap();

        let decision = compiled.check_path("common-grpc/client.go");
        assert!(decision.is_some());
        if let Some(ToolPermissionDecision::Deny(reason)) = decision {
            assert!(reason.contains("Update proto schemas"));
        } else {
            panic!("Expected Deny decision");
        }

        let allowed = compiled.check_path("services/user/main.go");
        assert!(allowed.is_none());
    }

    #[test]
    fn test_guardrails_command_whitelist() {
        let config = GuardrailConfig {
            workspace_id: "test".to_string(),
            version: "1.0".to_string(),
            deny_paths: vec![],
            path_rules: vec![],
            protected_branches: vec![],
            allowed_commands: vec!["cargo test".to_string(), "cargo check".to_string()],
            rag_service_url: None,
        };

        let compiled = CompiledGuardrails::compile(config).unwrap();

        assert!(compiled.check_terminal_command("cargo test --workspace").is_none());
        assert!(compiled.check_terminal_command("cargo check").is_none());

        let denied = compiled.check_terminal_command("rm -rf target");
        assert!(denied.is_some());
    }
}
