use anyhow::{Context as _, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct WindowUtilization {
    pub utilization_percent: u32,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ClaudeSubscriptionUsage {
    pub five_hour_window: Option<WindowUtilization>,
    pub seven_day_window: Option<WindowUtilization>,
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
    pub email_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CodexSubscriptionUsage {
    pub email_address: Option<String>,
    pub account_id: Option<String>,
    pub auth_mode: Option<String>,
    pub plan_tier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CopilotSubscriptionUsage {
    pub username: Option<String>,
    pub github_app_id: Option<String>,
    pub status: Option<String>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AntigravitySubscriptionUsage {
    pub email_address: Option<String>,
    pub account_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ProviderTokenMetrics {
    pub provider_name: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub total_tokens: u64,
    pub context_limit: u64,
    pub context_utilization_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AggregateUsageReport {
    pub claude_subscription: Option<ClaudeSubscriptionUsage>,
    pub codex_subscription: Option<CodexSubscriptionUsage>,
    pub copilot_subscription: Option<CopilotSubscriptionUsage>,
    pub antigravity_subscription: Option<AntigravitySubscriptionUsage>,
    pub providers: Vec<ProviderTokenMetrics>,
}

#[derive(Deserialize)]
struct ClaudeJsonUtilizationWindow {
    utilization: Option<u32>,
    resets_at: Option<String>,
}

#[derive(Deserialize)]
struct ClaudeJsonUtilization {
    five_hour: Option<ClaudeJsonUtilizationWindow>,
    seven_day: Option<ClaudeJsonUtilizationWindow>,
}

#[derive(Deserialize)]
struct ClaudeJsonCachedUsage {
    utilization: Option<ClaudeJsonUtilization>,
}

#[derive(Deserialize)]
struct ClaudeJsonOauthAccount {
    #[serde(rename = "emailAddress")]
    email_address: Option<String>,
    #[serde(rename = "seatTier")]
    seat_tier: Option<String>,
    #[serde(rename = "userRateLimitTier")]
    user_rate_limit_tier: Option<String>,
}

#[derive(Deserialize)]
struct ClaudeConfigFile {
    #[serde(rename = "cachedUsageUtilization")]
    cached_usage_utilization: Option<ClaudeJsonCachedUsage>,
    #[serde(rename = "oauthAccount")]
    oauth_account: Option<ClaudeJsonOauthAccount>,
}

static TOKEN_TRACKER: LazyLock<RwLock<HashMap<String, ProviderTokenMetrics>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn record_provider_tokens(
    provider_name: &str,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_write_tokens: u64,
    context_limit: u64,
) {
    let mut tracker = TOKEN_TRACKER.write();
    let entry = tracker
        .entry(provider_name.to_string())
        .or_insert_with(|| ProviderTokenMetrics {
            provider_name: provider_name.to_string(),
            context_limit: if context_limit > 0 { context_limit } else { 200_000 },
            ..Default::default()
        });

    entry.input_tokens = entry.input_tokens.saturating_add(input_tokens);
    entry.output_tokens = entry.output_tokens.saturating_add(output_tokens);
    entry.cache_read_tokens = entry.cache_read_tokens.saturating_add(cache_read_tokens);
    entry.cache_write_tokens = entry.cache_write_tokens.saturating_add(cache_write_tokens);
    entry.total_tokens = entry.input_tokens.saturating_add(entry.output_tokens);

    if entry.context_limit > 0 {
        entry.context_utilization_percent =
            ((entry.total_tokens as f64 / entry.context_limit as f64) * 100.0) as f32;
    }
}

pub fn read_claude_subscription_usage() -> Option<ClaudeSubscriptionUsage> {
    let home = paths::home_dir();
    let config_path = home.join(".claude.json");
    if !config_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&config_path).ok()?;
    parse_claude_usage_from_str(&content).ok()
}

pub fn parse_claude_usage_from_str(content: &str) -> Result<ClaudeSubscriptionUsage> {
    let parsed: ClaudeConfigFile = serde_json::from_str(content)
        .with_context(|| "Failed to parse claude configuration json")?;

    let mut usage = ClaudeSubscriptionUsage::default();

    if let Some(oauth) = parsed.oauth_account {
        usage.email_address = oauth.email_address;
        usage.subscription_type = oauth.seat_tier;
        usage.rate_limit_tier = oauth.user_rate_limit_tier;
    }

    if let Some(cached_usage) = parsed.cached_usage_utilization {
        if let Some(utilization) = cached_usage.utilization {
            if let Some(five_hour) = utilization.five_hour {
                if let Some(percent) = five_hour.utilization {
                    usage.five_hour_window = Some(WindowUtilization {
                        utilization_percent: percent,
                        resets_at: five_hour.resets_at,
                    });
                }
            }

            if let Some(seven_day) = utilization.seven_day {
                if let Some(percent) = seven_day.utilization {
                    usage.seven_day_window = Some(WindowUtilization {
                        utilization_percent: percent,
                        resets_at: seven_day.resets_at,
                    });
                }
            }
        }
    }

    Ok(usage)
}

pub fn read_codex_subscription_usage() -> Option<CodexSubscriptionUsage> {
    let home = paths::home_dir();
    let auth_path = home.join(".codex").join("auth.json");
    if !auth_path.exists() {
        return None;
    }
    let content = fs::read_to_string(&auth_path).ok()?;
    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    let tokens = val.get("tokens")?.as_object()?;
    let account_id = tokens
        .get("account_id")
        .and_then(|a| a.as_str())
        .map(|s| s.to_string());
    let auth_mode = val
        .get("auth_mode")
        .and_then(|a| a.as_str())
        .map(|s| s.to_string());
    let id_token = tokens
        .get("id_token")
        .and_then(|id| id.as_str())
        .unwrap_or("");
    let email = if !id_token.is_empty() {
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() >= 2 {
            use base64::Engine;
            base64::prelude::BASE64_STANDARD_NO_PAD
                .decode(parts[1])
                .or_else(|_| base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(parts[1]))
                .ok()
                .and_then(|payload| serde_json::from_slice::<serde_json::Value>(&payload).ok())
                .and_then(|v| {
                    v.get("email")
                        .and_then(|e| e.as_str())
                        .map(|s| s.to_string())
                })
        } else {
            None
        }
    } else {
        None
    };

    Some(CodexSubscriptionUsage {
        email_address: email,
        account_id,
        auth_mode,
        plan_tier: Some("ChatGPT Plus / Team (Codex CLI)".to_string()),
    })
}

pub fn read_copilot_subscription_usage() -> Option<CopilotSubscriptionUsage> {
    let home = paths::home_dir();
    let apps_path = home
        .join(".config")
        .join("github-copilot")
        .join("apps.json");
    if !apps_path.exists() {
        return None;
    }
    let content = fs::read_to_string(&apps_path).ok()?;
    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    let obj = val.as_object()?;
    for (_, entry) in obj {
        let user = entry
            .get("user")
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());
        let app_id = entry
            .get("githubAppId")
            .and_then(|a| a.as_str())
            .map(|s| s.to_string());
        let scopes = entry
            .get("scopes")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if user.is_some() || app_id.is_some() {
            return Some(CopilotSubscriptionUsage {
                username: user,
                github_app_id: app_id,
                status: Some("Active GitHub Copilot Subscription".to_string()),
                scopes,
            });
        }
    }
    None
}

pub fn read_antigravity_subscription_usage() -> Option<AntigravitySubscriptionUsage> {
    let home = paths::home_dir();
    let creds_path = home.join(".gemini").join("oauth_creds.json");
    if !creds_path.exists() {
        return None;
    }
    let content = fs::read_to_string(&creds_path).ok()?;
    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    let email = val
        .get("email")
        .or_else(|| val.get("user"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());
    let token_present = val.get("access_token").and_then(|t| t.as_str()).is_some();

    if token_present {
        Some(AntigravitySubscriptionUsage {
            email_address: email.or_else(|| Some("Active Antigravity Account".to_string())),
            account_type: Some("Google AI / Gemini Pro".to_string()),
            status: Some("Connected via Antigravity CLI OAuth".to_string()),
        })
    } else {
        None
    }
}

pub fn generate_usage_report() -> AggregateUsageReport {
    let claude_subscription = read_claude_subscription_usage();
    let codex_subscription = read_codex_subscription_usage();
    let copilot_subscription = read_copilot_subscription_usage();
    let antigravity_subscription = read_antigravity_subscription_usage();
    let tracker = TOKEN_TRACKER.read();
    let mut providers: Vec<ProviderTokenMetrics> = tracker.values().cloned().collect();

    // Ensure Claude, Codex, Copilot, and Antigravity entries exist for visibility
    if !providers.iter().any(|p| {
        p.provider_name.to_lowercase().contains("claude")
            || p.provider_name.to_lowercase().contains("anthropic")
    }) {
        providers.push(ProviderTokenMetrics {
            provider_name: "Claude (Anthropic)".to_string(),
            context_limit: 200_000,
            ..Default::default()
        });
    }

    if !providers.iter().any(|p| {
        p.provider_name.to_lowercase().contains("codex")
            || p.provider_name.to_lowercase().contains("openai")
    }) {
        providers.push(ProviderTokenMetrics {
            provider_name: "Codex (OpenAI)".to_string(),
            context_limit: 128_000,
            ..Default::default()
        });
    }

    if !providers.iter().any(|p| {
        p.provider_name.to_lowercase().contains("copilot")
            || p.provider_name.to_lowercase().contains("github")
    }) {
        providers.push(ProviderTokenMetrics {
            provider_name: "GitHub Copilot".to_string(),
            context_limit: 128_000,
            ..Default::default()
        });
    }

    if !providers.iter().any(|p| {
        p.provider_name.to_lowercase().contains("antigravity")
            || p.provider_name.to_lowercase().contains("gemini")
            || p.provider_name.to_lowercase().contains("google")
    }) {
        providers.push(ProviderTokenMetrics {
            provider_name: "Antigravity (Google AI)".to_string(),
            context_limit: 1_000_000,
            ..Default::default()
        });
    }

    providers.sort_by(|a, b| a.provider_name.cmp(&b.provider_name));

    AggregateUsageReport {
        claude_subscription,
        codex_subscription,
        copilot_subscription,
        antigravity_subscription,
        providers,
    }
}

pub fn format_usage_markdown(report: &AggregateUsageReport) -> String {
    let mut output = String::from("### 📊 Subscription & Token Usage Report\n\n");

    if let Some(claude) = &report.claude_subscription {
        output.push_str("#### 🟣 Claude (Anthropic Subscription)\n");
        if let Some(tier) = &claude.subscription_type {
            output.push_str(&format!("- **Plan Tier**: {}\n", tier));
        }
        if let Some(rate_tier) = &claude.rate_limit_tier {
            output.push_str(&format!("- **Rate Limit Tier**: {}\n", rate_tier));
        }
        if let Some(email) = &claude.email_address {
            output.push_str(&format!("- **Account**: {}\n", email));
        }
        if let Some(five_hour) = &claude.five_hour_window {
            output.push_str(&format!(
                "- **5-Hour Session Quota**: `{}%` used",
                five_hour.utilization_percent
            ));
            if let Some(resets) = &five_hour.resets_at {
                output.push_str(&format!(" (Resets at: {})", resets));
            }
            output.push('\n');
        }
        if let Some(seven_day) = &claude.seven_day_window {
            output.push_str(&format!(
                "- **7-Day Weekly Quota**: `{}%` used",
                seven_day.utilization_percent
            ));
            if let Some(resets) = &seven_day.resets_at {
                output.push_str(&format!(" (Resets at: {})", resets));
            }
            output.push('\n');
        }
        output.push('\n');
    }

    if let Some(codex) = &report.codex_subscription {
        output.push_str("#### 🟢 OpenAI Codex (ChatGPT Subscription)\n");
        if let Some(plan) = &codex.plan_tier {
            output.push_str(&format!("- **Plan Tier**: {}\n", plan));
        }
        if let Some(email) = &codex.email_address {
            output.push_str(&format!("- **Account**: {}\n", email));
        }
        if let Some(acc_id) = &codex.account_id {
            output.push_str(&format!("- **Account ID**: `{}`\n", acc_id));
        }
        output.push_str("- **Status**: Active Local Codex Subscription\n\n");
    }

    if let Some(copilot) = &report.copilot_subscription {
        output.push_str("#### 🔵 GitHub Copilot\n");
        if let Some(user) = &copilot.username {
            output.push_str(&format!("- **GitHub User**: {}\n", user));
        }
        if let Some(status) = &copilot.status {
            output.push_str(&format!("- **Status**: {}\n", status));
        }
        if !copilot.scopes.is_empty() {
            output.push_str(&format!("- **Scopes**: {}\n", copilot.scopes.join(", ")));
        }
        output.push('\n');
    }

    if let Some(antigravity) = &report.antigravity_subscription {
        output.push_str("#### 🔷 Google Antigravity (AI Studio / OAuth)\n");
        if let Some(email) = &antigravity.email_address {
            output.push_str(&format!("- **User Account**: {}\n", email));
        }
        if let Some(acc_type) = &antigravity.account_type {
            output.push_str(&format!("- **Platform**: {}\n", acc_type));
        }
        if let Some(status) = &antigravity.status {
            output.push_str(&format!("- **Status**: {}\n", status));
        }
        output.push('\n');
    }

    output.push_str("#### 📈 Active Token Metrics Per Provider\n");
    for provider in &report.providers {
        output.push_str(&format!("**{}**:\n", provider.provider_name));
        output.push_str(&format!("- Total Tokens: **{}**\n", provider.total_tokens));
        output.push_str(&format!("  - Input: {}\n", provider.input_tokens));
        output.push_str(&format!("  - Output: {}\n", provider.output_tokens));
        if provider.cache_read_tokens > 0 {
            output.push_str(&format!("  - Cache Reads: {} tokens\n", provider.cache_read_tokens));
        }
        output.push_str(&format!(
            "- Context Window: **{:.1}%** of {} tokens\n\n",
            provider.context_utilization_percent, provider.context_limit
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_claude_usage() {
        let sample_json = r#"{
            "oauthAccount": {
                "emailAddress": "user@example.com",
                "seatTier": "pro",
                "userRateLimitTier": "tier_2"
            },
            "cachedUsageUtilization": {
                "utilization": {
                    "five_hour": {
                        "utilization": 14,
                        "resets_at": "2026-08-06T05:59:59Z"
                    },
                    "seven_day": {
                        "utilization": 60,
                        "resets_at": "2026-08-07T01:59:59Z"
                    }
                }
            }
        }"#;

        let parsed = parse_claude_usage_from_str(sample_json).unwrap();
        assert_eq!(parsed.email_address.as_deref(), Some("user@example.com"));
        assert_eq!(parsed.subscription_type.as_deref(), Some("pro"));
        assert_eq!(parsed.rate_limit_tier.as_deref(), Some("tier_2"));
        assert_eq!(parsed.five_hour_window.unwrap().utilization_percent, 14);
        assert_eq!(parsed.seven_day_window.unwrap().utilization_percent, 60);
    }

    #[test]
    fn test_record_provider_tokens() {
        record_provider_tokens("Claude", 1000, 200, 500, 100, 200_000);
        let report = generate_usage_report();
        let claude = report.providers.iter().find(|p| p.provider_name == "Claude");
        assert!(claude.is_some());
        let claude = claude.unwrap();
        assert_eq!(claude.input_tokens, 1000);
        assert_eq!(claude.output_tokens, 200);
        assert_eq!(claude.total_tokens, 1200);
    }
}
