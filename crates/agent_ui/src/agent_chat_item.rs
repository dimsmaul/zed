use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, Render,
    SharedString, Window, div,
};
use ui::prelude::*;
use workspace::item::Item;
use workspace::Workspace;

use crate::AgentPanel;

pub struct AgentChatItem {
    panel: Entity<AgentPanel>,
}

impl AgentChatItem {
    pub fn new(panel: Entity<AgentPanel>) -> Self {
        Self { panel }
    }

    pub fn panel(&self) -> &Entity<AgentPanel> {
        &self.panel
    }

    pub fn open_in_workspace(
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        for pane in workspace.panes() {
            let item_found = pane.read(cx).items().find_map(|item| item.downcast::<AgentChatItem>());
            if let Some(item) = item_found {
                if let Some(index) = pane.read(cx).index_for_item(&item) {
                    pane.update(cx, |pane, cx| {
                        pane.activate_item(index, true, true, window, cx);
                    });
                    return;
                }
            }
        }

        if let Some(agent_panel) = workspace.panel::<AgentPanel>(cx) {
            let item = cx.new(|_| AgentChatItem::new(agent_panel));
            workspace.add_item_to_active_pane(Box::new(item), None, true, window, cx);
        }
    }
}

impl Focusable for AgentChatItem {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.panel.read(cx).focus_handle(cx)
    }
}

impl EventEmitter<()> for AgentChatItem {}

impl Render for AgentChatItem {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.panel.clone())
    }
}

impl Item for AgentChatItem {
    type Event = ();

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        "Agent".into()
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ZedAssistant))
    }
}
