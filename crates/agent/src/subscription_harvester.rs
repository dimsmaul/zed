use anyhow::Result;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct HarvestedClaudeSession {
    pub access_token: String,
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
}

#[derive(Deserialize)]
struct ClaudeOauthPayload {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "subscriptionType")]
    subscription_type: Option<String>,
    #[serde(rename = "rateLimitTier")]
    rate_limit_tier: Option<String>,
}

#[derive(Deserialize)]
struct KeychainCredentialsWrapper {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<ClaudeOauthPayload>,
}

pub fn harvest_claude_session() -> Option<HarvestedClaudeSession> {
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        if !key.trim().is_empty() {
            return Some(HarvestedClaudeSession {
                access_token: key,
                subscription_type: Some("api_key".to_string()),
                rate_limit_tier: None,
            });
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(session) = harvest_from_macos_keychain() {
            return Some(session);
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn harvest_from_macos_keychain() -> Option<HarvestedClaudeSession> {
    // Attempt reading from known service names used by the Claude Code CLI
    let possible_services = [
        "Claude Code-credentials-516b0843",
        "Claude Code-credentials",
        "Claude Safe Storage",
    ];

    for service_name in possible_services {
        if let Ok(output) = Command::new("security")
            .args(["find-generic-password", "-s", service_name, "-w"])
            .output()
        {
            if output.status.success() {
                let raw_secret = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if let Ok(wrapper) = serde_json::from_str::<KeychainCredentialsWrapper>(&raw_secret) {
                    if let Some(oauth) = wrapper.claude_ai_oauth {
                        if !oauth.access_token.is_empty() {
                            return Some(HarvestedClaudeSession {
                                access_token: oauth.access_token,
                                subscription_type: oauth.subscription_type,
                                rate_limit_tier: oauth.rate_limit_tier,
                            });
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harvest_claude_session_runs_safely() {
        // Function must safely return Some or None without panicking
        let _session = harvest_claude_session();
    }
}
