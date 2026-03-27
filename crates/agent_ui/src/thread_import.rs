use std::sync::Arc;

use collections::HashSet;
use gpui::{
    App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, Render, SharedString,
    Task, WeakEntity, Window,
};

use picker::{Picker, PickerDelegate};
use project::{AgentId, AgentRegistryStore, AgentServerStore};
use ui::{Checkbox, CommonAnimationExt as _, KeyBinding, ListItem, ListItemSpacing, prelude::*};
use util::ResultExt;
use workspace::ModalView;

#[derive(Clone)]
struct AgentEntry {
    agent_id: AgentId,
    display_name: SharedString,
    icon_path: Option<SharedString>,
}

pub struct ThreadImportModal {
    picker: Entity<Picker<ThreadImportPickerDelegate>>,
    checked_agents: HashSet<AgentId>,
    is_importing: bool,
}

impl ThreadImportModal {
    pub fn new(
        agent_server_store: Entity<AgentServerStore>,
        agent_registry_store: Entity<AgentRegistryStore>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let agent_entries = agent_server_store
            .read(cx)
            .external_agents()
            .map(|agent_id| {
                let display_name = agent_server_store
                    .read(cx)
                    .agent_display_name(agent_id)
                    .or_else(|| {
                        agent_registry_store
                            .read(cx)
                            .agent(agent_id)
                            .map(|agent| agent.name().clone())
                    })
                    .unwrap_or_else(|| agent_id.0.clone());
                let icon_path = agent_server_store
                    .read(cx)
                    .agent_icon(agent_id)
                    .or_else(|| {
                        agent_registry_store
                            .read(cx)
                            .agent(agent_id)
                            .and_then(|agent| agent.icon_path().cloned())
                    });

                AgentEntry {
                    agent_id: agent_id.clone(),
                    display_name,
                    icon_path,
                }
            })
            .collect::<Vec<_>>();

        let thread_import_modal = cx.entity().downgrade();
        let picker: Entity<Picker<ThreadImportPickerDelegate>> = cx.new(|cx| {
            Picker::uniform_list(
                ThreadImportPickerDelegate::new(thread_import_modal, agent_entries),
                window,
                cx,
            )
        });

        Self {
            picker,
            checked_agents: HashSet::default(),
            is_importing: false,
        }
    }

    fn set_agent_checked(&mut self, agent_id: AgentId, state: ToggleState, cx: &mut Context<Self>) {
        match state {
            ToggleState::Selected => {
                self.checked_agents.insert(agent_id);
            }
            ToggleState::Unselected | ToggleState::Indeterminate => {
                self.checked_agents.remove(&agent_id);
            }
        }
        cx.notify();
    }

    fn toggle_agent_checked(&mut self, agent_id: AgentId, cx: &mut Context<Self>) {
        if self.checked_agents.contains(&agent_id) {
            self.checked_agents.remove(&agent_id);
        } else {
            self.checked_agents.insert(agent_id);
        }
        cx.notify();
    }

    fn import_threads(&mut self, _: &menu::Confirm, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_importing {
            return;
        }

        self.is_importing = true;
        cx.notify();
    }
}

impl EventEmitter<DismissEvent> for ThreadImportModal {}

impl Focusable for ThreadImportModal {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.picker.focus_handle(cx)
    }
}

impl ModalView for ThreadImportModal {}

impl Render for ThreadImportModal {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("ThreadImportModal")
            .w(rems(34.))
            .child(self.picker.clone())
    }
}

struct ThreadImportPickerDelegate {
    thread_import_modal: WeakEntity<ThreadImportModal>,
    agent_entries: Vec<AgentEntry>,
    filtered_indices: Vec<usize>,
    selected_index: usize,
}

impl ThreadImportPickerDelegate {
    fn new(
        thread_import_modal: WeakEntity<ThreadImportModal>,
        agent_entries: Vec<AgentEntry>,
    ) -> Self {
        let filtered_indices = (0..agent_entries.len()).collect();
        Self {
            thread_import_modal,
            agent_entries,
            filtered_indices,
            selected_index: 0,
        }
    }
}

impl PickerDelegate for ThreadImportPickerDelegate {
    type ListItem = AnyElement;

    fn placeholder_text(&self, _window: &mut Window, _cx: &mut App) -> Arc<str> {
        "Search ACP agents…".into()
    }

    fn no_matches_text(&self, _window: &mut Window, _cx: &mut App) -> Option<SharedString> {
        Some(if self.agent_entries.is_empty() {
            "No ACP agents available.".into()
        } else {
            "No ACP agents match your search.".into()
        })
    }

    fn match_count(&self) -> usize {
        self.filtered_indices.len()
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(
        &mut self,
        ix: usize,
        _window: &mut Window,
        _cx: &mut Context<Picker<Self>>,
    ) {
        self.selected_index = ix.min(self.filtered_indices.len().saturating_sub(1));
    }

    fn update_matches(
        &mut self,
        query: String,
        _window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let query = query.to_lowercase();
        self.filtered_indices = self
            .agent_entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                query.is_empty() || entry.display_name.to_lowercase().contains(&query)
            })
            .map(|(index, _)| index)
            .collect();
        self.selected_index = self
            .selected_index
            .min(self.filtered_indices.len().saturating_sub(1));
        cx.notify();
        Task::ready(())
    }

    fn confirm(&mut self, secondary: bool, window: &mut Window, cx: &mut Context<Picker<Self>>) {
        if secondary {
            self.thread_import_modal
                .update(cx, |thread_import_modal, cx| {
                    if thread_import_modal.is_importing {
                        return;
                    }

                    thread_import_modal.import_threads(&menu::Confirm, window, cx);
                })
                .log_err();
            return;
        }

        let Some(entry) = self
            .filtered_indices
            .get(self.selected_index)
            .and_then(|index| self.agent_entries.get(*index))
        else {
            return;
        };

        self.thread_import_modal
            .update(cx, |thread_import_modal, cx| {
                thread_import_modal.toggle_agent_checked(entry.agent_id.clone(), cx);
            })
            .log_err();
    }

    fn dismissed(&mut self, _window: &mut Window, cx: &mut Context<Picker<Self>>) {
        self.thread_import_modal
            .update(cx, |_thread_import_modal, cx| cx.emit(DismissEvent))
            .log_err();
    }

    fn render_match(
        &self,
        ix: usize,
        selected: bool,
        _window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        let entry = self.agent_entries.get(*self.filtered_indices.get(ix)?)?;
        let is_checked = self
            .thread_import_modal
            .read_with(cx, |modal, _cx| {
                modal.checked_agents.contains(&entry.agent_id)
            })
            .ok()
            .unwrap_or(false);

        Some(
            ListItem::new(("thread-import-agent", ix))
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(selected)
                .start_slot(
                    Checkbox::new(
                        ("thread-import-agent-checkbox", ix),
                        if is_checked {
                            ToggleState::Selected
                        } else {
                            ToggleState::Unselected
                        },
                    )
                    .on_click({
                        let thread_import_modal = self.thread_import_modal.clone();
                        let agent_id = entry.agent_id.clone();
                        move |state, _window, cx| {
                            thread_import_modal
                                .update(cx, |thread_import_modal, cx| {
                                    thread_import_modal.set_agent_checked(
                                        agent_id.clone(),
                                        *state,
                                        cx,
                                    );
                                })
                                .log_err();
                        }
                    }),
                )
                .child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .child(if let Some(icon_path) = entry.icon_path.clone() {
                            Icon::from_external_svg(icon_path)
                                .color(Color::Muted)
                                .size(IconSize::Small)
                        } else {
                            Icon::new(IconName::Sparkle)
                                .color(Color::Muted)
                                .size(IconSize::Small)
                        })
                        .child(Label::new(entry.display_name.clone())),
                )
                .into_any_element(),
        )
    }

    fn render_footer(
        &self,
        _window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Option<AnyElement> {
        let is_importing = self
            .thread_import_modal
            .read_with(cx, |thread_import_modal, _cx| {
                thread_import_modal.is_importing
            })
            .ok()
            .unwrap_or(false);

        Some(
            h_flex()
                .w_full()
                .p_1p5()
                .gap_2()
                .justify_end()
                .border_t_1()
                .border_color(cx.theme().colors().border_variant)
                .when(is_importing, |this| {
                    this.child(
                        Icon::new(IconName::ArrowCircle)
                            .size(IconSize::Small)
                            .color(Color::Muted)
                            .with_rotate_animation(2),
                    )
                })
                .child(
                    Button::new("import-threads", "Import Threads")
                        .disabled(is_importing)
                        .key_binding(
                            KeyBinding::for_action(&menu::SecondaryConfirm, cx)
                                .map(|kb| kb.size(rems_from_px(12.))),
                        )
                        .on_click({
                            let thread_import_modal = self.thread_import_modal.clone();
                            move |_, window, cx| {
                                thread_import_modal
                                    .update(cx, |thread_import_modal, cx| {
                                        if thread_import_modal.is_importing {
                                            return;
                                        }

                                        thread_import_modal.import_threads(
                                            &menu::Confirm,
                                            window,
                                            cx,
                                        );
                                    })
                                    .log_err();
                            }
                        }),
                )
                .into_any_element(),
        )
    }
}
