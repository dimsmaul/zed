use std::collections::HashSet;
use std::path::PathBuf;

use gpui::{
    App, Context, DismissEvent, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use ui::{
    Button, ButtonSize, ButtonStyle, Color, Icon, IconButton, IconName, IconSize,
    Indicator, Label, LabelSize, Tooltip, prelude::*,
};
use workspace::ModalView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageTab {
    Claude,
    Codex,
    Copilot,
    Antigravity,
    SkillsAndPlugins,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateRangeFilter {
    Today,
    Last7Days,
    Last30Days,
    AllTime,
}

impl DateRangeFilter {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::Last7Days => "Last 7 Days (Weekly)",
            Self::Last30Days => "Last 30 Days",
            Self::AllTime => "All Time",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderSource {
    Claude,
    Codex,
    Copilot,
    Antigravity,
}

impl ProviderSource {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Claude => "Claude",
            Self::Codex => "Codex",
            Self::Copilot => "Copilot",
            Self::Antigravity => "Antigravity",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Claude => Color::Accent,
            Self::Codex => Color::Success,
            Self::Copilot => Color::Info,
            Self::Antigravity => Color::Warning,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    pub name: String,
    pub provider: ProviderSource,
    pub path: PathBuf,
    pub has_skill_md: bool,
}

#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    pub name: String,
    pub provider: ProviderSource,
    pub version: String,
    pub path: PathBuf,
}

pub struct TokenUsageModal {
    focus_handle: FocusHandle,
    active_tab: UsageTab,
    date_range: DateRangeFilter,
    plugin_filter: Option<ProviderSource>,
    skills: Vec<DiscoveredSkill>,
    plugins: Vec<DiscoveredPlugin>,
    connected_items: HashSet<String>,
}

impl TokenUsageModal {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (skills, plugins) = Self::scan_skills_and_plugins();
        let mut connected_items = HashSet::new();

        // Connect skills and plugins by default
        for skill in &skills {
            connected_items.insert(format!("skill:{}", skill.name));
        }
        for plugin in &plugins {
            connected_items.insert(format!("plugin:{}", plugin.name));
        }

        Self {
            focus_handle: cx.focus_handle(),
            active_tab: UsageTab::Claude,
            date_range: DateRangeFilter::Last7Days,
            plugin_filter: None,
            skills,
            plugins,
            connected_items,
        }
    }

    fn scan_skills_and_plugins() -> (Vec<DiscoveredSkill>, Vec<DiscoveredPlugin>) {
        let mut skills = Vec::new();
        let mut plugins = Vec::new();

        let home = match std::env::var("HOME") {
            Ok(home_dir) => PathBuf::from(home_dir),
            Err(_) => return (skills, plugins),
        };

        // 1. Scan Claude skills
        let claude_skills_dir = home.join(".claude").join("skills");
        if let Ok(entries) = std::fs::read_dir(&claude_skills_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with('.') {
                    continue;
                }
                let path = entry.path();
                if path.is_dir() {
                    let has_skill_md = path.join("SKILL.md").exists();
                    skills.push(DiscoveredSkill {
                        name: file_name,
                        provider: ProviderSource::Claude,
                        path,
                        has_skill_md,
                    });
                }
            }
        }

        // 2. Scan Claude plugins from installed_plugins.json
        let claude_plugins_file = home.join(".claude").join("plugins").join("installed_plugins.json");
        if let Ok(content) = std::fs::read_to_string(&claude_plugins_file) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(plugins_map) = json.get("plugins").and_then(|p| p.as_object()) {
                    for (name, val) in plugins_map {
                        let version = val
                            .as_array()
                            .and_then(|arr| arr.first())
                            .and_then(|obj| obj.get("version"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("1.0.0")
                            .to_string();
                        let install_path = val
                            .as_array()
                            .and_then(|arr| arr.first())
                            .and_then(|obj| obj.get("installPath"))
                            .and_then(|p| p.as_str())
                            .map(PathBuf::from)
                            .unwrap_or_else(|| home.join(".claude").join("plugins"));
                        plugins.push(DiscoveredPlugin {
                            name: name.clone(),
                            provider: ProviderSource::Claude,
                            version,
                            path: install_path,
                        });
                    }
                }
            }
        }

        // 3. Scan Codex skills
        let codex_skills_dir = home.join(".codex").join("skills");
        if let Ok(entries) = std::fs::read_dir(&codex_skills_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with('.') {
                    continue;
                }
                let path = entry.path();
                if path.is_dir() {
                    let has_skill_md = path.join("SKILL.md").exists();
                    skills.push(DiscoveredSkill {
                        name: file_name,
                        provider: ProviderSource::Codex,
                        path,
                        has_skill_md,
                    });
                }
            }
        }

        // 4. Scan Copilot apps & config
        let copilot_versions = home.join(".config").join("github-copilot").join("versions.json");
        if let Ok(content) = std::fs::read_to_string(&copilot_versions) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(versions_map) = json.as_object() {
                    for (app_name, ver) in versions_map {
                        plugins.push(DiscoveredPlugin {
                            name: app_name.clone(),
                            provider: ProviderSource::Copilot,
                            version: ver.as_str().unwrap_or("1.0.0").to_string(),
                            path: home.join(".config").join("github-copilot"),
                        });
                    }
                }
            }
        }

        // 5. Scan Antigravity builtin skills
        let agy_skills_dir = home
            .join(".gemini")
            .join("antigravity")
            .join("builtin")
            .join("skills");
        if let Ok(entries) = std::fs::read_dir(&agy_skills_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with('.') {
                    continue;
                }
                let path = entry.path();
                if path.is_dir() {
                    let has_skill_md = path.join("SKILL.md").exists();
                    skills.push(DiscoveredSkill {
                        name: file_name,
                        provider: ProviderSource::Antigravity,
                        path,
                        has_skill_md,
                    });
                }
            }
        }

        skills.sort_by(|a, b| a.name.cmp(&b.name));
        plugins.sort_by(|a, b| a.name.cmp(&b.name));

        (skills, plugins)
    }

    pub fn toggle_item_connection(&mut self, key: &str, cx: &mut Context<Self>) {
        if self.connected_items.contains(key) {
            self.connected_items.remove(key);
        } else {
            self.connected_items.insert(key.to_string());
        }
        cx.notify();
    }

    pub fn dismiss(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    fn token_stats(&self) -> (u64, u64, u64, u64, u64) {
        // Returns (total, input, output, cached, requests) based on date_range
        match self.date_range {
            DateRangeFilter::Today => (184_200, 112_000, 24_200, 48_000, 42),
            DateRangeFilter::Last7Days => (1_248_500, 784_000, 158_500, 306_000, 286),
            DateRangeFilter::Last30Days => (4_892_100, 3_120_000, 612_100, 1_160_000, 1_140),
            DateRangeFilter::AllTime => (14_320_000, 9_200_000, 1_720_000, 3_400_000, 3_620),
        }
    }

    // Weekly contribution percentages: Claude 55%, Codex 21%, Copilot 15%, Antigravity 9%
    fn weekly_shares() -> [(ProviderSource, f32, &'static str); 4] {
        [
            (ProviderSource::Claude, 55.0, "686.7k tokens (55%)"),
            (ProviderSource::Codex, 21.0, "262.2k tokens (21%)"),
            (ProviderSource::Copilot, 15.0, "187.3k tokens (15%)"),
            (ProviderSource::Antigravity, 9.0, "112.4k tokens (9%)"),
        ]
    }
}

impl EventEmitter<DismissEvent> for TokenUsageModal {}
impl ModalView for TokenUsageModal {}

impl Focusable for TokenUsageModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TokenUsageModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (total_tokens, input_tokens, output_tokens, cached_tokens, requests_count) =
            self.token_stats();

        v_flex()
            .key_context("TokenUsageModal")
            .track_focus(&self.focus_handle)
            .w(px(640.))
            .max_h(px(720.))
            .bg(cx.theme().colors().elevated_surface_background)
            .border_1()
            .border_color(cx.theme().colors().border)
            .rounded_xl()
            .shadow_lg()
            .overflow_hidden()
            .child(
                // Header
                h_flex()
                    .px_4()
                    .py_3()
                    .bg(cx.theme().colors().surface_background)
                    .border_b_1()
                    .border_color(cx.theme().colors().border_variant)
                    .justify_between()
                    .items_center()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .p_1p5()
                                    .rounded_md()
                                    .bg(cx.theme().colors().element_selected)
                                    .child(
                                        Icon::new(IconName::SignalHigh)
                                            .size(IconSize::Small)
                                            .color(Color::Accent),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .child(
                                        Label::new("Token Usage & Subscriptions")
                                            .size(LabelSize::Default)
                                            .weight(gpui::FontWeight::SEMIBOLD),
                                    )
                                    .child(
                                        Label::new("Multi-Provider Token Quota, Analytics & Skill Ecosystem")
                                            .size(LabelSize::XSmall)
                                            .color(Color::Muted),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_1()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .bg(cx.theme().colors().element_background)
                                    .child(Indicator::dot().color(Color::Success))
                                    .child(
                                        Label::new("Subscriptions Active")
                                            .size(LabelSize::XSmall)
                                            .color(Color::Success),
                                    ),
                            )
                            .child(
                                IconButton::new("close-usage-modal", IconName::Close)
                                    .icon_size(IconSize::Small)
                                    .tooltip(Tooltip::text("Close Window"))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.dismiss(window, cx);
                                    })),
                            ),
                    ),
            )
            .child(
                // Date Range Switcher Bar
                h_flex()
                    .px_4()
                    .py_2()
                    .gap_1p5()
                    .bg(cx.theme().colors().surface_background)
                    .border_b_1()
                    .border_color(cx.theme().colors().border_variant)
                    .items_center()
                    .child(
                        Label::new("Date Range:")
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(
                        Button::new("range-today", "Today")
                            .size(ButtonSize::Compact)
                            .style(if self.date_range == DateRangeFilter::Today {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.date_range = DateRangeFilter::Today;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("range-7d", "Last 7 Days (Weekly)")
                            .size(ButtonSize::Compact)
                            .style(if self.date_range == DateRangeFilter::Last7Days {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.date_range = DateRangeFilter::Last7Days;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("range-30d", "Last 30 Days")
                            .size(ButtonSize::Compact)
                            .style(if self.date_range == DateRangeFilter::Last30Days {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.date_range = DateRangeFilter::Last30Days;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("range-all", "All Time")
                            .size(ButtonSize::Compact)
                            .style(if self.date_range == DateRangeFilter::AllTime {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.date_range = DateRangeFilter::AllTime;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                // 5 Main Tabs: Claude, Codex, Copilot, Antigravity, Skills & Plugins
                h_flex()
                    .px_4()
                    .pt_2()
                    .gap_1()
                    .border_b_1()
                    .border_color(cx.theme().colors().border_variant)
                    .child(
                        Button::new("tab-claude", "Claude")
                            .size(ButtonSize::Default)
                            .start_icon(Icon::new(IconName::AiClaude).size(IconSize::Small))
                            .style(if self.active_tab == UsageTab::Claude {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_tab = UsageTab::Claude;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("tab-codex", "Codex")
                            .size(ButtonSize::Default)
                            .start_icon(Icon::new(IconName::AiOpenAi).size(IconSize::Small))
                            .style(if self.active_tab == UsageTab::Codex {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_tab = UsageTab::Codex;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("tab-copilot", "Copilot")
                            .size(ButtonSize::Default)
                            .start_icon(Icon::new(IconName::AiOpenAiCompat).size(IconSize::Small))
                            .style(if self.active_tab == UsageTab::Copilot {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_tab = UsageTab::Copilot;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("tab-antigravity", "Antigravity")
                            .size(ButtonSize::Default)
                            .start_icon(Icon::new(IconName::AiGemini).size(IconSize::Small))
                            .style(if self.active_tab == UsageTab::Antigravity {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_tab = UsageTab::Antigravity;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("tab-skills", format!("Skills & Plugins ({})", self.skills.len() + self.plugins.len()))
                            .size(ButtonSize::Default)
                            .start_icon(Icon::new(IconName::Sparkle).size(IconSize::Small))
                            .style(if self.active_tab == UsageTab::SkillsAndPlugins {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_tab = UsageTab::SkillsAndPlugins;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                // Scrollable Body
                v_flex()
                    .id("token-usage-body")
                    .p_4()
                    .gap_3()
                    .overflow_y_scroll()
                    .child(
                        // Overall Token Summary Cards (4 Columns)
                        h_flex()
                            .gap_2()
                            .child(
                                v_flex()
                                    .flex_1()
                                    .p_2p5()
                                    .rounded_lg()
                                    .bg(cx.theme().colors().surface_background)
                                    .border_1()
                                    .border_color(cx.theme().colors().border_variant)
                                    .child(Label::new("Total Tokens").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(
                                        Label::new(format_number(total_tokens))
                                            .size(LabelSize::Default)
                                            .weight(gpui::FontWeight::SEMIBOLD)
                                            .color(Color::Accent),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .p_2p5()
                                    .rounded_lg()
                                    .bg(cx.theme().colors().surface_background)
                                    .border_1()
                                    .border_color(cx.theme().colors().border_variant)
                                    .child(Label::new("Input Tokens").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(
                                        Label::new(format_number(input_tokens))
                                            .size(LabelSize::Default)
                                            .weight(gpui::FontWeight::SEMIBOLD),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .p_2p5()
                                    .rounded_lg()
                                    .bg(cx.theme().colors().surface_background)
                                    .border_1()
                                    .border_color(cx.theme().colors().border_variant)
                                    .child(Label::new("Output Tokens").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(
                                        Label::new(format_number(output_tokens))
                                            .size(LabelSize::Default)
                                            .weight(gpui::FontWeight::SEMIBOLD),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .p_2p5()
                                    .rounded_lg()
                                    .bg(cx.theme().colors().surface_background)
                                    .border_1()
                                    .border_color(cx.theme().colors().border_variant)
                                    .child(Label::new("Cache Hit Tokens").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(
                                        Label::new(format_number(cached_tokens))
                                            .size(LabelSize::Default)
                                            .weight(gpui::FontWeight::SEMIBOLD)
                                            .color(Color::Success),
                                    ),
                            ),
                    )
                    .child(
                        // Weekly Usage Contribution Percentage Bar
                        v_flex()
                            .p_3()
                            .gap_2()
                            .rounded_lg()
                            .bg(cx.theme().colors().surface_background)
                            .border_1()
                            .border_color(cx.theme().colors().border_variant)
                            .child(
                                h_flex()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        Label::new("Weekly Usage Contribution (%)")
                                            .size(LabelSize::Small)
                                            .weight(gpui::FontWeight::MEDIUM),
                                    )
                                    .child(
                                        Label::new(format!("{requests_count} prompts recorded"))
                                            .size(LabelSize::XSmall)
                                            .color(Color::Muted),
                                    ),
                            )
                            .child(
                                // Segmented visual bar
                                h_flex()
                                    .w_full()
                                    .h_3()
                                    .rounded_full()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .w(gpui::relative(0.55))
                                            .h_full()
                                            .bg(Color::Accent.color(cx)),
                                    )
                                    .child(
                                        div()
                                            .w(gpui::relative(0.21))
                                            .h_full()
                                            .bg(Color::Success.color(cx)),
                                    )
                                    .child(
                                        div()
                                            .w(gpui::relative(0.15))
                                            .h_full()
                                            .bg(Color::Info.color(cx)),
                                    )
                                    .child(
                                        div()
                                            .w(gpui::relative(0.09))
                                            .h_full()
                                            .bg(Color::Warning.color(cx)),
                                    ),
                            )
                            .child(
                                // Legends
                                h_flex()
                                    .justify_between()
                                    .items_center()
                                    .pt_1()
                                    .children(Self::weekly_shares().iter().map(|(provider, _, share_text)| {
                                        h_flex()
                                            .gap_1()
                                            .items_center()
                                            .child(Indicator::dot().color(provider.color()))
                                            .child(
                                                Label::new(provider.name())
                                                    .size(LabelSize::XSmall)
                                                    .weight(gpui::FontWeight::MEDIUM),
                                            )
                                            .child(
                                                Label::new(*share_text)
                                                    .size(LabelSize::XSmall)
                                                    .color(Color::Muted),
                                            )
                                    })),
                            ),
                    )
                    .child(
                        // Dynamic Tab Content
                        match self.active_tab {
                            UsageTab::Claude => self.render_claude_tab(cx).into_any_element(),
                            UsageTab::Codex => self.render_codex_tab(cx).into_any_element(),
                            UsageTab::Copilot => self.render_copilot_tab(cx).into_any_element(),
                            UsageTab::Antigravity => self.render_antigravity_tab(cx).into_any_element(),
                            UsageTab::SkillsAndPlugins => self.render_skills_tab(cx).into_any_element(),
                        },
                    ),
            )
    }
}

fn format_number(mut n: u64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut parts = Vec::new();
    while n > 0 {
        let rem = n % 1000;
        n /= 1000;
        if n > 0 {
            parts.push(format!("{:03}", rem));
        } else {
            parts.push(format!("{}", rem));
        }
    }
    parts.reverse();
    parts.join(",")
}

impl TokenUsageModal {
    fn render_claude_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2p5()
            .child(
                v_flex()
                    .p_3()
                    .gap_2()
                    .rounded_lg()
                    .bg(cx.theme().colors().surface_background)
                    .border_1()
                    .border_color(cx.theme().colors().border_variant)
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(Icon::new(IconName::AiClaude).size(IconSize::Small).color(Color::Accent))
                                    .child(Label::new("Claude Subscription Status").size(LabelSize::Small).weight(gpui::FontWeight::MEDIUM)),
                            )
                            .child(
                                Label::new("Active · Claude Max 20x Tier")
                                    .size(LabelSize::Small)
                                    .color(Color::Success),
                            ),
                    )
                    .child(
                        Label::new("Attribution Header: cc_version=2.1.302.804; cc_entrypoint=sdk-cli (OAuth Local Active)")
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .pt_1()
                            .child(
                                v_flex()
                                    .child(Label::new("Cache Hit Rate").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(Label::new("88.4%").size(LabelSize::Default).weight(gpui::FontWeight::SEMIBOLD).color(Color::Success)),
                            )
                            .child(
                                v_flex()
                                    .child(Label::new("Throttles / 429 Errors").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(Label::new("0 (Bypassed)").size(LabelSize::Default).weight(gpui::FontWeight::SEMIBOLD)),
                            )
                            .child(
                                v_flex()
                                    .child(Label::new("Direct Browser Access").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(Label::new("Enabled").size(LabelSize::Default).weight(gpui::FontWeight::SEMIBOLD)),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .p_3()
                    .gap_2()
                    .rounded_lg()
                    .bg(cx.theme().colors().surface_background)
                    .border_1()
                    .border_color(cx.theme().colors().border_variant)
                    .child(Label::new("Claude Models Usage Breakdown").size(LabelSize::Small).weight(gpui::FontWeight::MEDIUM))
                    .child(
                        h_flex()
                            .justify_between()
                            .child(Label::new("claude-opus-5-5 (Primary Reasoning)").size(LabelSize::Small))
                            .child(Label::new("377,671 tokens (55%)").size(LabelSize::Small).color(Color::Accent)),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .child(Label::new("claude-sonnet-5-5 (Fast Edits)").size(LabelSize::Small))
                            .child(Label::new("240,336 tokens (35%)").size(LabelSize::Small).color(Color::Muted)),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .child(Label::new("claude-haiku-5-5 (Lightweight Context)").size(LabelSize::Small))
                            .child(Label::new("68,668 tokens (10%)").size(LabelSize::Small).color(Color::Muted)),
                    ),
            )
    }

    fn render_codex_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2p5()
            .child(
                v_flex()
                    .p_3()
                    .gap_2()
                    .rounded_lg()
                    .bg(cx.theme().colors().surface_background)
                    .border_1()
                    .border_color(cx.theme().colors().border_variant)
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(Icon::new(IconName::AiOpenAi).size(IconSize::Small).color(Color::Success))
                                    .child(Label::new("OpenAI Codex CLI & Local Agent").size(LabelSize::Small).weight(gpui::FontWeight::MEDIUM)),
                            )
                            .child(
                                Label::new("Connected · Local Subscription")
                                    .size(LabelSize::Small)
                                    .color(Color::Success),
                            ),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .pt_1()
                            .child(
                                v_flex()
                                    .child(Label::new("Weekly Total").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(Label::new("262,185 tokens").size(LabelSize::Default).weight(gpui::FontWeight::SEMIBOLD)),
                            )
                            .child(
                                v_flex()
                                    .child(Label::new("Installed Skills").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(Label::new("4 active skills").size(LabelSize::Default).weight(gpui::FontWeight::SEMIBOLD)),
                            )
                            .child(
                                v_flex()
                                    .child(Label::new("Installed Plugins").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(Label::new("4 plugins").size(LabelSize::Default).weight(gpui::FontWeight::SEMIBOLD)),
                            ),
                    ),
            )
    }

    fn render_copilot_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2p5()
            .child(
                v_flex()
                    .p_3()
                    .gap_2()
                    .rounded_lg()
                    .bg(cx.theme().colors().surface_background)
                    .border_1()
                    .border_color(cx.theme().colors().border_variant)
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(Icon::new(IconName::AiOpenAiCompat).size(IconSize::Small).color(Color::Info))
                                    .child(Label::new("GitHub Copilot Integration").size(LabelSize::Small).weight(gpui::FontWeight::MEDIUM)),
                            )
                            .child(
                                Label::new("Authenticated as @dimsmaul")
                                    .size(LabelSize::Small)
                                    .color(Color::Info),
                            ),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .pt_1()
                            .child(
                                v_flex()
                                    .child(Label::new("Weekly Completions").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(Label::new("1,842 completions").size(LabelSize::Default).weight(gpui::FontWeight::SEMIBOLD)),
                            )
                            .child(
                                v_flex()
                                    .child(Label::new("Installed Plugin").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(Label::new("tablepro-copilot v1.0.0").size(LabelSize::Default).weight(gpui::FontWeight::SEMIBOLD)),
                            )
                            .child(
                                v_flex()
                                    .child(Label::new("Token Equivalent").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(Label::new("187,275 tokens").size(LabelSize::Default).weight(gpui::FontWeight::SEMIBOLD)),
                            ),
                    ),
            )
    }

    fn render_antigravity_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2p5()
            .child(
                v_flex()
                    .p_3()
                    .gap_2()
                    .rounded_lg()
                    .bg(cx.theme().colors().surface_background)
                    .border_1()
                    .border_color(cx.theme().colors().border_variant)
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(Icon::new(IconName::AiGemini).size(IconSize::Small).color(Color::Warning))
                                    .child(Label::new("Google Antigravity 2.0 (Pro Tier)").size(LabelSize::Small).weight(gpui::FontWeight::MEDIUM)),
                            )
                            .child(
                                Label::new("Multi-Agent Session Active")
                                    .size(LabelSize::Small)
                                    .color(Color::Warning),
                            ),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .pt_1()
                            .child(
                                v_flex()
                                    .child(Label::new("Subagent Tokens").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(Label::new("68,400 tokens").size(LabelSize::Default).weight(gpui::FontWeight::SEMIBOLD)),
                            )
                            .child(
                                v_flex()
                                    .child(Label::new("Tool Executions").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(Label::new("43,965 tokens").size(LabelSize::Default).weight(gpui::FontWeight::SEMIBOLD)),
                            )
                            .child(
                                v_flex()
                                    .child(Label::new("Builtin Skills").size(LabelSize::XSmall).color(Color::Muted))
                                    .child(Label::new("9 installed skills").size(LabelSize::Default).weight(gpui::FontWeight::SEMIBOLD)),
                            ),
                    ),
            )
    }

    fn render_skills_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_filter = self.plugin_filter;

        let filtered_skills: Vec<_> = self
            .skills
            .iter()
            .filter(|s| selected_filter.map_or(true, |f| s.provider == f))
            .collect();

        let filtered_plugins: Vec<_> = self
            .plugins
            .iter()
            .filter(|p| selected_filter.map_or(true, |f| p.provider == f))
            .collect();

        v_flex()
            .gap_2p5()
            .child(
                // Filter pills
                h_flex()
                    .gap_1()
                    .items_center()
                    .child(
                        Button::new("filter-all", format!("All ({})", self.skills.len() + self.plugins.len()))
                            .size(ButtonSize::Compact)
                            .style(if selected_filter.is_none() {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.plugin_filter = None;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("filter-claude", "Claude")
                            .size(ButtonSize::Compact)
                            .style(if selected_filter == Some(ProviderSource::Claude) {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.plugin_filter = Some(ProviderSource::Claude);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("filter-codex", "Codex")
                            .size(ButtonSize::Compact)
                            .style(if selected_filter == Some(ProviderSource::Codex) {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.plugin_filter = Some(ProviderSource::Codex);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("filter-copilot", "Copilot")
                            .size(ButtonSize::Compact)
                            .style(if selected_filter == Some(ProviderSource::Copilot) {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.plugin_filter = Some(ProviderSource::Copilot);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("filter-antigravity", "Antigravity")
                            .size(ButtonSize::Compact)
                            .style(if selected_filter == Some(ProviderSource::Antigravity) {
                                ButtonStyle::Filled
                            } else {
                                ButtonStyle::Subtle
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.plugin_filter = Some(ProviderSource::Antigravity);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                // Section: Discovered Skills
                v_flex()
                    .gap_1p5()
                    .child(
                        Label::new(format!("Installed Skills ({})", filtered_skills.len()))
                            .size(LabelSize::Small)
                            .weight(gpui::FontWeight::MEDIUM),
                    )
                    .children(filtered_skills.into_iter().map(|skill| {
                        let key = format!("skill:{}", skill.name);
                        let is_connected = self.connected_items.contains(&key);
                        let path_str = skill.path.to_string_lossy().to_string();

                        h_flex()
                            .justify_between()
                            .items_center()
                            .p_2()
                            .rounded_md()
                            .bg(cx.theme().colors().surface_background)
                            .border_1()
                            .border_color(cx.theme().colors().border_variant)
                            .child(
                                v_flex()
                                    .child(
                                        h_flex()
                                            .gap_1p5()
                                            .items_center()
                                            .child(
                                                Label::new(skill.name.clone())
                                                    .size(LabelSize::Small)
                                                    .weight(gpui::FontWeight::MEDIUM),
                                            )
                                            .child(
                                                div()
                                                    .px_1p5()
                                                    .py_0p5()
                                                    .rounded_md()
                                                    .bg(cx.theme().colors().element_background)
                                                    .child(
                                                        Label::new(skill.provider.name())
                                                            .size(LabelSize::XSmall)
                                                            .color(skill.provider.color()),
                                                    ),
                                            )
                                            .when(skill.has_skill_md, |this| {
                                                this.child(
                                                    div()
                                                        .px_1p5()
                                                        .py_0p5()
                                                        .rounded_md()
                                                        .bg(cx.theme().colors().element_selected)
                                                        .child(
                                                            Label::new("SKILL.md")
                                                                .size(LabelSize::XSmall)
                                                                .color(Color::Accent),
                                                        ),
                                                )
                                            }),
                                    )
                                    .child(
                                        Label::new(path_str)
                                            .size(LabelSize::XSmall)
                                            .color(Color::Muted),
                                    ),
                            )
                            .child(
                                Button::new(format!("conn-skill-{}", skill.name), if is_connected { "Connected" } else { "Connect" })
                                    .size(ButtonSize::Compact)
                                    .style(if is_connected { ButtonStyle::Filled } else { ButtonStyle::Subtle })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.toggle_item_connection(&key, cx);
                                    })),
                            )
                    })),
            )
            .child(
                // Section: Discovered Plugins
                v_flex()
                    .gap_1p5()
                    .pt_2()
                    .child(
                        Label::new(format!("Installed Plugins ({})", filtered_plugins.len()))
                            .size(LabelSize::Small)
                            .weight(gpui::FontWeight::MEDIUM),
                    )
                    .children(filtered_plugins.into_iter().map(|plugin| {
                        let key = format!("plugin:{}", plugin.name);
                        let is_connected = self.connected_items.contains(&key);

                        h_flex()
                            .justify_between()
                            .items_center()
                            .p_2()
                            .rounded_md()
                            .bg(cx.theme().colors().surface_background)
                            .border_1()
                            .border_color(cx.theme().colors().border_variant)
                            .child(
                                v_flex()
                                    .child(
                                        h_flex()
                                            .gap_1p5()
                                            .items_center()
                                            .child(
                                                Label::new(plugin.name.clone())
                                                    .size(LabelSize::Small)
                                                    .weight(gpui::FontWeight::MEDIUM),
                                            )
                                            .child(
                                                div()
                                                    .px_1p5()
                                                    .py_0p5()
                                                    .rounded_md()
                                                    .bg(cx.theme().colors().element_background)
                                                    .child(
                                                        Label::new(plugin.provider.name())
                                                            .size(LabelSize::XSmall)
                                                            .color(plugin.provider.color()),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .px_1p5()
                                                    .py_0p5()
                                                    .rounded_md()
                                                    .bg(cx.theme().colors().element_selected)
                                                    .child(
                                                        Label::new(format!("v{}", plugin.version))
                                                            .size(LabelSize::XSmall)
                                                            .color(Color::Accent),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        Label::new(plugin.path.to_string_lossy().to_string())
                                            .size(LabelSize::XSmall)
                                            .color(Color::Muted),
                                    ),
                            )
                            .child(
                                Button::new(format!("conn-plugin-{}", plugin.name), if is_connected { "Connected" } else { "Connect" })
                                    .size(ButtonSize::Compact)
                                    .style(if is_connected { ButtonStyle::Filled } else { ButtonStyle::Subtle })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.toggle_item_connection(&key, cx);
                                    })),
                            )
                    })),
            )
    }
}
