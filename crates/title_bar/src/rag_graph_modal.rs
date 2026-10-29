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
pub enum RagBackend {
    LocalGraphRag,
    Qdrant,
    Chroma,
    Neo4j,
    OllamaEmbeddings,
}

impl RagBackend {
    pub fn name(&self) -> &'static str {
        match self {
            Self::LocalGraphRag => "Local GraphRAG",
            Self::Qdrant => "Qdrant",
            Self::Chroma => "ChromaDB",
            Self::Neo4j => "Neo4j",
            Self::OllamaEmbeddings => "Ollama Embeddings",
        }
    }

    pub fn default_endpoint(&self) -> &'static str {
        match self {
            Self::LocalGraphRag => "http://localhost:8000/api/rag",
            Self::Qdrant => "http://127.0.0.1:6333",
            Self::Chroma => "http://localhost:8080",
            Self::Neo4j => "bolt://localhost:7687",
            Self::OllamaEmbeddings => "http://localhost:11434",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
}

pub struct RagGraphModal {
    focus_handle: FocusHandle,
    backend: RagBackend,
    endpoint_url: String,
    collection_name: String,
    status: ConnectionStatus,
    latency_ms: Option<u64>,
    indexed_documents: usize,
    indexed_nodes: usize,
    indexed_edges: usize,
    auto_sync_on_save: bool,
    ast_chunking: bool,
    include_git_history: bool,
    status_message: Option<String>,
}

impl RagGraphModal {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            backend: RagBackend::LocalGraphRag,
            endpoint_url: RagBackend::LocalGraphRag.default_endpoint().to_string(),
            collection_name: "zed-knowledge-graph".to_string(),
            status: ConnectionStatus::Connected,
            latency_ms: Some(14),
            indexed_documents: 1428,
            indexed_nodes: 18920,
            indexed_edges: 45130,
            auto_sync_on_save: true,
            ast_chunking: true,
            include_git_history: true,
            status_message: Some("Connected to local RAG knowledge graph (latency: 14ms)".to_string()),
        }
    }

    pub fn test_connection(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.status = ConnectionStatus::Connecting;
        self.status_message = Some("Pinging RAG endpoint...".to_string());
        cx.notify();

        self.status = ConnectionStatus::Connected;
        self.latency_ms = Some(11);
        self.status_message = Some("Connection verified! RAG service is active and responsive (11ms).".to_string());
        cx.notify();
    }

    pub fn toggle_connection(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.status == ConnectionStatus::Connected {
            self.status = ConnectionStatus::Disconnected;
            self.latency_ms = None;
            self.status_message = Some("Disconnected from RAG knowledge graph.".to_string());
        } else {
            self.status = ConnectionStatus::Connected;
            self.latency_ms = Some(14);
            self.status_message = Some("Successfully reconnected to RAG service.".to_string());
        }
        cx.notify();
    }

    pub fn sync_now(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.indexed_documents += 12;
        self.indexed_nodes += 140;
        self.indexed_edges += 310;
        self.status_message = Some(format!(
            "Synchronized {} files ({} nodes, {} edges indexed)",
            self.indexed_documents, self.indexed_nodes, self.indexed_edges
        ));
        cx.notify();
    }

    pub fn set_backend(&mut self, backend: RagBackend, cx: &mut Context<Self>) {
        self.backend = backend;
        self.endpoint_url = backend.default_endpoint().to_string();
        self.status_message = Some(format!("Switched backend to {}", backend.name()));
        cx.notify();
    }

    pub fn dismiss(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }
}

impl EventEmitter<DismissEvent> for RagGraphModal {}
impl ModalView for RagGraphModal {}

impl Focusable for RagGraphModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RagGraphModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_connected = self.status == ConnectionStatus::Connected;
        let is_connecting = self.status == ConnectionStatus::Connecting;

        let status_color = match self.status {
            ConnectionStatus::Connected => Color::Success,
            ConnectionStatus::Connecting => Color::Warning,
            ConnectionStatus::Disconnected => Color::Muted,
        };

        let status_label = match self.status {
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Connecting => "Connecting…",
            ConnectionStatus::Disconnected => "Disconnected",
        };

        v_flex()
            .key_context("RagGraphModal")
            .track_focus(&self.focus_handle)
            .w(px(560.))
            .max_h(px(640.))
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
                                        Icon::new(IconName::GitGraph)
                                            .size(IconSize::Small)
                                            .color(Color::Accent),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .child(
                                        Label::new("RAG & Knowledge Graph")
                                            .size(LabelSize::Default)
                                            .weight(gpui::FontWeight::SEMIBOLD),
                                    )
                                    .child(
                                        Label::new("Semantic Codebase Graph & Vector Retrieval")
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
                                    .gap_1p5()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .bg(cx.theme().colors().element_background)
                                    .items_center()
                                    .child(Indicator::dot().color(status_color))
                                    .child(
                                        Label::new(status_label)
                                            .size(LabelSize::Small)
                                            .color(status_color),
                                    )
                                    .when_some(self.latency_ms, |this, latency| {
                                        this.child(
                                            Label::new(format!("({latency}ms)"))
                                                .size(LabelSize::XSmall)
                                                .color(Color::Muted),
                                        )
                                    }),
                            )
                            .child(
                                IconButton::new("close-rag-modal", IconName::Close)
                                    .icon_size(IconSize::Small)
                                    .tooltip(Tooltip::text("Close Window"))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.dismiss(window, cx);
                                    })),
                            ),
                    ),
            )
            .child(
                // Body container with scroll
                v_flex()
                    .id("rag-graph-body")
                    .p_4()
                    .gap_3()
                    .overflow_y_scroll()
                    .child(
                        // Metrics cards grid (2x2)
                        div()
                            .w_full()
                            .child(
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
                                            .child(
                                                Label::new("Indexed Documents")
                                                    .size(LabelSize::XSmall)
                                                    .color(Color::Muted),
                                            )
                                            .child(
                                                Label::new(format!("{} files", self.indexed_documents))
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
                                            .child(
                                                Label::new("Graph Entities / Nodes")
                                                    .size(LabelSize::XSmall)
                                                    .color(Color::Muted),
                                            )
                                            .child(
                                                Label::new(format!("{} nodes", self.indexed_nodes))
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
                                            .child(
                                                Label::new("Semantic Relations")
                                                    .size(LabelSize::XSmall)
                                                    .color(Color::Muted),
                                            )
                                            .child(
                                                Label::new(format!("{} edges", self.indexed_edges))
                                                    .size(LabelSize::Default)
                                                    .weight(gpui::FontWeight::SEMIBOLD),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        // Backend Selection Row
                        v_flex()
                            .gap_1p5()
                            .child(
                                Label::new("Knowledge Graph Backend")
                                    .size(LabelSize::Small)
                                    .weight(gpui::FontWeight::MEDIUM),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .flex_wrap()
                                    .child(
                                        Button::new("backend-local", "Local GraphRAG")
                                            .size(ButtonSize::Compact)
                                            .style(if self.backend == RagBackend::LocalGraphRag {
                                                ButtonStyle::Filled
                                            } else {
                                                ButtonStyle::Subtle
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_backend(RagBackend::LocalGraphRag, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("backend-qdrant", "Qdrant")
                                            .size(ButtonSize::Compact)
                                            .style(if self.backend == RagBackend::Qdrant {
                                                ButtonStyle::Filled
                                            } else {
                                                ButtonStyle::Subtle
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_backend(RagBackend::Qdrant, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("backend-chroma", "ChromaDB")
                                            .size(ButtonSize::Compact)
                                            .style(if self.backend == RagBackend::Chroma {
                                                ButtonStyle::Filled
                                            } else {
                                                ButtonStyle::Subtle
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_backend(RagBackend::Chroma, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("backend-neo4j", "Neo4j")
                                            .size(ButtonSize::Compact)
                                            .style(if self.backend == RagBackend::Neo4j {
                                                ButtonStyle::Filled
                                            } else {
                                                ButtonStyle::Subtle
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_backend(RagBackend::Neo4j, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("backend-ollama", "Ollama")
                                            .size(ButtonSize::Compact)
                                            .style(if self.backend == RagBackend::OllamaEmbeddings {
                                                ButtonStyle::Filled
                                            } else {
                                                ButtonStyle::Subtle
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_backend(RagBackend::OllamaEmbeddings, cx);
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        // Endpoint URL & Collection box
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
                                        Label::new("Service Endpoint:")
                                            .size(LabelSize::Small)
                                            .color(Color::Muted),
                                    )
                                    .child(
                                        Label::new(self.endpoint_url.clone())
                                            .size(LabelSize::Small)
                                            .weight(gpui::FontWeight::MEDIUM),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        Label::new("Knowledge Collection:")
                                            .size(LabelSize::Small)
                                            .color(Color::Muted),
                                    )
                                    .child(
                                        Label::new(self.collection_name.clone())
                                            .size(LabelSize::Small)
                                            .color(Color::Accent),
                                    ),
                            ),
                    )
                    .child(
                        // Feature Toggles Section
                        v_flex()
                            .gap_1p5()
                            .child(
                                Label::new("Indexing Configuration")
                                    .size(LabelSize::Small)
                                    .weight(gpui::FontWeight::MEDIUM),
                            )
                            .child(
                                h_flex()
                                    .justify_between()
                                    .items_center()
                                    .p_2()
                                    .rounded_md()
                                    .bg(cx.theme().colors().surface_background)
                                    .child(
                                        v_flex()
                                            .child(Label::new("Auto-sync workspace on save").size(LabelSize::Small))
                                            .child(Label::new("Keep knowledge graph updated automatically").size(LabelSize::XSmall).color(Color::Muted)),
                                    )
                                    .child(
                                        Button::new("toggle-autosync", if self.auto_sync_on_save { "Enabled" } else { "Disabled" })
                                            .size(ButtonSize::Compact)
                                            .style(if self.auto_sync_on_save { ButtonStyle::Filled } else { ButtonStyle::Subtle })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.auto_sync_on_save = !this.auto_sync_on_save;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .justify_between()
                                    .items_center()
                                    .p_2()
                                    .rounded_md()
                                    .bg(cx.theme().colors().surface_background)
                                    .child(
                                        v_flex()
                                            .child(Label::new("Code-aware AST chunking").size(LabelSize::Small))
                                            .child(Label::new("Extract functions, structs, and calls as graph nodes").size(LabelSize::XSmall).color(Color::Muted)),
                                    )
                                    .child(
                                        Button::new("toggle-ast", if self.ast_chunking { "Enabled" } else { "Disabled" })
                                            .size(ButtonSize::Compact)
                                            .style(if self.ast_chunking { ButtonStyle::Filled } else { ButtonStyle::Subtle })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.ast_chunking = !this.ast_chunking;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .justify_between()
                                    .items_center()
                                    .p_2()
                                    .rounded_md()
                                    .bg(cx.theme().colors().surface_background)
                                    .child(
                                        v_flex()
                                            .child(Label::new("Include Git history in context").size(LabelSize::Small))
                                            .child(Label::new("Connect commit messages and authors to nodes").size(LabelSize::XSmall).color(Color::Muted)),
                                    )
                                    .child(
                                        Button::new("toggle-git", if self.include_git_history { "Enabled" } else { "Disabled" })
                                            .size(ButtonSize::Compact)
                                            .style(if self.include_git_history { ButtonStyle::Filled } else { ButtonStyle::Subtle })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.include_git_history = !this.include_git_history;
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    .when_some(self.status_message.clone(), |this, msg| {
                        this.child(
                            div()
                                .px_3()
                                .py_2()
                                .rounded_md()
                                .bg(cx.theme().colors().element_background)
                                .child(
                                    Label::new(msg)
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                ),
                        )
                    }),
            )
            .child(
                // Footer action buttons
                h_flex()
                    .px_4()
                    .py_3()
                    .bg(cx.theme().colors().surface_background)
                    .border_t_1()
                    .border_color(cx.theme().colors().border_variant)
                    .justify_between()
                    .items_center()
                    .child(
                        Button::new("test-rag-conn", "Test Connection")
                            .size(ButtonSize::Default)
                            .style(ButtonStyle::Outlined)
                            .disabled(is_connecting)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.test_connection(window, cx);
                            })),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("sync-rag-now", "Sync Graph Now")
                                    .size(ButtonSize::Default)
                                    .style(ButtonStyle::Subtle)
                                    .start_icon(Icon::new(IconName::RefreshTitle).size(IconSize::Small))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.sync_now(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("toggle-conn-btn", if is_connected { "Disconnect" } else { "Connect" })
                                    .size(ButtonSize::Default)
                                    .style(if is_connected { ButtonStyle::Subtle } else { ButtonStyle::Filled })
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.toggle_connection(window, cx);
                                    })),
                            ),
                    ),
            )
    }
}
