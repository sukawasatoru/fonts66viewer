/*
 * Copyright 2026 sukawasatoru
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use crate::asset::Asset;
use crate::data::font_list::FontListRepository;
use crate::data::preferences::PreferencesRepository;
use crate::feature::settings::checkable_font_list_item::checkable_font_list_item;
use crate::feature::settings::font_list_item::FontListItem;
use crate::model::{
    DEFAULT_SAMPLE_FONT_SIZE, FontEntry, Preferences, Preset, SAVE_PREFS_DEBOUNCE_MILLIS,
    TOOLBAR_HEIGHT, WINDOW_BODY_MARGIN, XMessage,
};
use crate::prelude::*;
use crate::widget::settings_button_solid;
use iced::widget::container::background;
use iced::widget::operation;
use iced::widget::rule::horizontal;
use iced::widget::{
    button, column, container, radio, row, scrollable, space, svg, text_editor, text_input,
};
use iced::{Alignment, Color, Element, Length, Subscription, Task, Theme, padding};
use iced_aw::number_input;
use indexmap::IndexMap;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};

static RENAME_INPUT_ID: LazyLock<iced::widget::Id> =
    LazyLock::new(|| iced::widget::Id::from("preset-rename-input"));

#[derive(Clone, Debug)]
pub enum SettingsViewCommand {
    FontListItemChanged(FontEntry, bool),
    FontSizeUpdated(u32),
    PrefsLoaded(Preferences),
    PresetAddClicked,
    PresetDeleteClicked(String),
    PresetMoveDown(String),
    PresetMoveUp(String),
    PresetRenameStarted(String),
    PresetRenameChanged(String),
    PresetRenameConfirmed,
    PresetSelected(String),
    SavePrefsRequested(u64),
    SettingsButtonClicked,
    TextEditorAction(text_editor::Action),
    SendXMessage(XMessage),
    Sink,
    XMessage(XMessage),
}

pub struct SettingsView {
    custom_text_content: text_editor::Content,
    font_size: u32,
    font_list_item_map: IndexMap<String, FontListItem>,
    prefs: Option<Preferences>,
    save_prefs_version: u64,
    prefs_repo: Arc<PreferencesRepository>,
    prefs_selected_name: Option<String>,
    editing_preset: Option<EditingPreset>,
}

impl SettingsView {
    pub fn new(
        font_list_repo: Arc<FontListRepository>,
        prefs_repo: Arc<PreferencesRepository>,
    ) -> Self {
        let font_entries = font_list_repo.find_all();
        let mut font_list_item_map = IndexMap::with_capacity(font_entries.len());
        for font_entry in font_entries {
            font_list_item_map.insert(
                font_entry.filepath.to_owned(),
                FontListItem {
                    enabled: true,
                    font_entry,
                },
            );
        }

        Self {
            custom_text_content: text_editor::Content::new(),
            font_size: DEFAULT_SAMPLE_FONT_SIZE,
            font_list_item_map,
            prefs: None,
            save_prefs_version: 0,
            prefs_repo,
            prefs_selected_name: None,
            editing_preset: None,
        }
    }

    pub fn update(&mut self, command: SettingsViewCommand) -> Task<SettingsViewCommand> {
        match command {
            SettingsViewCommand::FontListItemChanged(font_entry, enabled) => {
                let font_list_item = self
                    .font_list_item_map
                    .get_mut(&font_entry.filepath)
                    .expect("SettingsView should have entry");
                font_list_item.enabled = enabled;

                if let Some(preset) = self.selected_preset_mut() {
                    if enabled {
                        preset.enable_paths.push(font_entry.filepath.clone());
                    } else {
                        preset
                            .enable_paths
                            .retain(|path| path != &font_entry.filepath);
                    }
                }

                Task::batch([
                    send_xmessage(XMessage::FontEntries(self.create_font_entries())),
                    self.schedule_save_prefs(),
                ])
            }
            SettingsViewCommand::FontSizeUpdated(size) => {
                self.font_size = size;

                if let Some(preset) = self.selected_preset_mut() {
                    preset.font_size = size;
                }

                Task::batch([
                    send_xmessage(XMessage::FontSize(size)),
                    self.schedule_save_prefs(),
                ])
            }
            SettingsViewCommand::PresetAddClicked => {
                let Some(prefs) = self.prefs.as_mut() else {
                    return Task::none();
                };

                let new_name = next_preset_name(&prefs.presets);
                let new_preset = Preset {
                    name: new_name.clone(),
                    font_size: DEFAULT_SAMPLE_FONT_SIZE,
                    enable_paths: self.font_list_item_map.keys().cloned().collect(),
                };
                prefs.presets.push(new_preset.clone());
                self.prefs_selected_name = Some(new_name);
                self.apply_preset(&new_preset);

                Task::batch([self.notify_preset_applied(), self.schedule_save_prefs()])
            }
            SettingsViewCommand::PresetMoveUp(name) => self.move_preset(&name, -1),
            SettingsViewCommand::PresetMoveDown(name) => self.move_preset(&name, 1),
            SettingsViewCommand::PresetDeleteClicked(name) => {
                let Some(prefs) = self.prefs.as_mut() else {
                    return Task::none();
                };

                if prefs.presets.len() <= 1 {
                    return Task::none();
                }

                let Some(index) = prefs.presets.iter().position(|p| p.name == name) else {
                    return Task::none();
                };

                let was_selected = self.prefs_selected_name.as_ref() == Some(&name);
                prefs.presets.remove(index);

                if was_selected {
                    let new_index = index.min(prefs.presets.len() - 1);
                    let new_preset = prefs.presets[new_index].clone();
                    self.prefs_selected_name = Some(new_preset.name.clone());
                    self.apply_preset(&new_preset);

                    Task::batch([self.notify_preset_applied(), self.schedule_save_prefs()])
                } else {
                    self.schedule_save_prefs()
                }
            }
            SettingsViewCommand::PresetRenameStarted(name) => {
                self.editing_preset = Some(EditingPreset {
                    original_name: name.clone(),
                    new_name: name,
                });
                operation::focus(RENAME_INPUT_ID.clone())
            }
            SettingsViewCommand::PresetRenameChanged(new_name) => {
                if let Some(editing) = self.editing_preset.as_mut() {
                    editing.new_name = new_name;
                }
                Task::none()
            }
            SettingsViewCommand::PresetRenameConfirmed => {
                let Some(editing) = self.editing_preset.take() else {
                    return Task::none();
                };

                let new_name = editing.new_name.trim().to_string();
                if new_name.is_empty() || new_name == editing.original_name {
                    return Task::none();
                }

                let Some(prefs) = self.prefs.as_mut() else {
                    return Task::none();
                };

                if prefs.presets.iter().any(|p| p.name == new_name) {
                    return Task::none();
                }

                if let Some(preset) = prefs
                    .presets
                    .iter_mut()
                    .find(|p| p.name == editing.original_name)
                {
                    preset.name = new_name.clone();
                }

                if self.prefs_selected_name.as_ref() == Some(&editing.original_name) {
                    self.prefs_selected_name = Some(new_name);
                }

                self.schedule_save_prefs()
            }
            SettingsViewCommand::PresetSelected(name) => {
                if self.prefs_selected_name.as_ref() == Some(&name) {
                    return Task::none();
                }

                self.prefs_selected_name = Some(name.clone());

                if let Some(preset) = self
                    .prefs
                    .as_ref()
                    .and_then(|p| p.presets.iter().find(|preset| preset.name == name))
                    .cloned()
                {
                    self.apply_preset(&preset);
                }

                Task::batch([self.notify_preset_applied(), self.schedule_save_prefs()])
            }
            SettingsViewCommand::PrefsLoaded(mut prefs) => {
                if prefs.presets.is_empty() {
                    prefs.presets.push(Preset {
                        name: "Preset 1".into(),
                        font_size: DEFAULT_SAMPLE_FONT_SIZE,
                        enable_paths: self.font_list_item_map.keys().cloned().collect(),
                    });
                }

                if self.prefs_selected_name.is_none() {
                    self.prefs_selected_name =
                        prefs.presets.first().map(|preset| preset.name.clone());
                }

                if let Some(selected_name) = &self.prefs_selected_name
                    && let Some(preset) = prefs.presets.iter().find(|p| &p.name == selected_name)
                {
                    self.apply_preset(preset);
                }

                self.prefs = Some(prefs);

                self.notify_preset_applied()
            }
            SettingsViewCommand::SavePrefsRequested(version) => {
                if version != self.save_prefs_version {
                    return Task::none();
                }
                // Reset to 0 to indicate no pending unsaved changes.
                self.save_prefs_version = 0;

                let Some(prefs) = self.prefs.clone() else {
                    return Task::none();
                };
                let prefs_repo = self.prefs_repo.clone();
                Task::perform(
                    async move {
                        match prefs_repo.save(prefs) {
                            Ok(_) => info!("saved preferences"),
                            Err(e) => warn!(?e, "failed to save preferences"),
                        };
                    },
                    |_| SettingsViewCommand::Sink,
                )
            }
            SettingsViewCommand::SettingsButtonClicked => send_xmessage(XMessage::SettingsClose),
            SettingsViewCommand::TextEditorAction(action) => {
                let need_update = matches!(&action, text_editor::Action::Edit(_));

                self.custom_text_content.perform(action);

                if need_update {
                    send_xmessage(XMessage::CustomText(self.custom_text_content.text()))
                } else {
                    Task::none()
                }
            }
            // Propagate to App layer via Task so it can be converted to AppCommand::XMessage.
            SettingsViewCommand::SendXMessage(data) => send_xmessage(data),
            SettingsViewCommand::Sink => Task::none(),
            SettingsViewCommand::XMessage(message) => match message {
                // Save preferences synchronously on close. Since the app uses
                // exit_on_close_request(false), this handler runs before
                // window::close(id) is executed by the app layer.
                XMessage::CloseRequested(_) => {
                    if self.save_prefs_version > 0 {
                        // Reset to 0 to indicate no pending unsaved changes.
                        self.save_prefs_version = 0;
                        if let Some(prefs) = &self.prefs {
                            let _ = self.prefs_repo.save(prefs.clone());
                        }
                    }
                    Task::none()
                }
                // Load preferences asynchronously via Task::perform. The result
                // is delivered as PrefsLoaded, which creates a default preset if
                // needed and sends FontSize/FontEntries messages.
                XMessage::Init => {
                    let prefs_repo = self.prefs_repo.clone();
                    Task::perform(
                        async move { prefs_repo.retrieve() },
                        |result| match result {
                            Ok(prefs) => SettingsViewCommand::PrefsLoaded(prefs),
                            Err(e) => {
                                warn!(?e, "Failed to load preferences");
                                SettingsViewCommand::PrefsLoaded(Preferences { presets: vec![] })
                            }
                        },
                    )
                }
                _ => Task::none(),
            },
        }
    }

    pub fn subscription(&self) -> Subscription<SettingsViewCommand> {
        Subscription::none()
    }

    pub fn view(&self) -> Element<'_, SettingsViewCommand> {
        let mut content = column![
            container(settings_button_solid(
                SettingsViewCommand::SettingsButtonClicked
            ))
            .padding(padding::right(WINDOW_BODY_MARGIN))
            .align_right(Length::Fill)
            .center_y(TOOLBAR_HEIGHT),
        ];

        let mut content_inner = self
            .view_presets()
            .push(divider())
            .push(
                container(
                    text_editor(&self.custom_text_content)
                        .placeholder("Custom text...")
                        .on_action(SettingsViewCommand::TextEditorAction),
                )
                .height(73),
            )
            .push(divider())
            .push(
                row![
                    "Font size:",
                    space().width(Length::Fill),
                    number_input(
                        &self.font_size,
                        1..=1000,
                        SettingsViewCommand::FontSizeUpdated
                    )
                    .width(87),
                ]
                .align_y(Alignment::Center),
            )
            .push(divider());

        for item in self.font_list_item_map.values() {
            content_inner = content_inner.push(checkable_font_list_item(item));
        }

        content = content.push(scrollable(content_inner.padding(15).width(Length::Fill)));

        container(content)
            .style(settings_view_style)
            .width(268)
            .height(Length::Fill)
            .into()
    }

    fn view_presets(&self) -> iced::widget::Column<'_, SettingsViewCommand> {
        let mut content = column!["Preset:"];

        let Some(prefs) = &self.prefs else {
            return content;
        };

        let preset_count = prefs.presets.len();
        let is_editing = self.editing_preset.is_some();
        for preset in &prefs.presets {
            let editing_this = self
                .editing_preset
                .as_ref()
                .is_some_and(|e| e.original_name == preset.name);

            let mut preset_row = if editing_this {
                let new_name = self.editing_preset.as_ref().unwrap().new_name.clone();
                row![
                    text_input("", &new_name)
                        .id(RENAME_INPUT_ID.clone())
                        .on_input(SettingsViewCommand::PresetRenameChanged)
                        .on_submit(SettingsViewCommand::PresetRenameConfirmed),
                ]
            } else {
                let name = preset.name.clone();
                row![radio(
                    &preset.name,
                    &preset.name,
                    self.prefs_selected_name.as_ref(),
                    move |_| SettingsViewCommand::PresetSelected(name.clone()),
                )]
            }
            .align_y(Alignment::Center);

            preset_row = preset_row.push(space().width(Length::Fill));

            if editing_this {
                preset_row = preset_row.push(
                    preset_action_btn("\u{2713}")
                        .on_press(SettingsViewCommand::PresetRenameConfirmed),
                );
            } else if !is_editing {
                let name = preset.name.clone();
                preset_row = preset_row.push(
                    preset_action_btn(pencil_icon())
                        .on_press(SettingsViewCommand::PresetRenameStarted(name)),
                );
            }

            let mut delete_btn = preset_action_btn(trash_icon());
            if preset_count > 1 && !is_editing {
                let name = preset.name.clone();
                delete_btn = delete_btn.on_press(SettingsViewCommand::PresetDeleteClicked(name));
            }
            preset_row = preset_row.push(delete_btn);

            content = content.push(preset_row);
        }

        let selected_index = self
            .prefs_selected_name
            .as_ref()
            .and_then(|name| prefs.presets.iter().position(|p| &p.name == name));

        let mut toolbar_row =
            row![button("+ New").on_press(SettingsViewCommand::PresetAddClicked),].spacing(4);

        if !is_editing {
            let mut up_btn = button("\u{25B2} Up");
            if let Some(idx) = selected_index
                && idx > 0
            {
                let name = prefs.presets[idx].name.clone();
                up_btn = up_btn.on_press(SettingsViewCommand::PresetMoveUp(name));
            }
            toolbar_row = toolbar_row.push(up_btn);

            let mut down_btn = button("\u{25BC} Down");
            if let Some(idx) = selected_index
                && idx < preset_count - 1
            {
                let name = prefs.presets[idx].name.clone();
                down_btn = down_btn.on_press(SettingsViewCommand::PresetMoveDown(name));
            }
            toolbar_row = toolbar_row.push(down_btn);
        }

        content.push(toolbar_row)
    }

    // Debounced save: increment save_prefs_version and wait
    // SAVE_PREFS_DEBOUNCE_MILLIS before firing SavePrefsRequested. Only the
    // request whose version matches the current save_prefs_version will
    // actually persist, so rapid changes result in a single save.
    fn schedule_save_prefs(&mut self) -> Task<SettingsViewCommand> {
        self.save_prefs_version += 1;
        let version = self.save_prefs_version;
        Task::perform(
            async move {
                tokio::time::sleep(std::time::Duration::from_millis(SAVE_PREFS_DEBOUNCE_MILLIS))
                    .await;
            },
            move |_| SettingsViewCommand::SavePrefsRequested(version),
        )
    }

    fn notify_preset_applied(&self) -> Task<SettingsViewCommand> {
        Task::batch([
            send_xmessage(XMessage::FontSize(self.font_size)),
            send_xmessage(XMessage::FontEntries(self.create_font_entries())),
        ])
    }

    fn move_preset(&mut self, name: &str, direction: i32) -> Task<SettingsViewCommand> {
        let Some(prefs) = self.prefs.as_mut() else {
            return Task::none();
        };
        let Some(index) = prefs.presets.iter().position(|p| p.name == name) else {
            return Task::none();
        };
        let new_index = index as i32 + direction;
        if new_index < 0 || new_index >= prefs.presets.len() as i32 {
            return Task::none();
        }
        prefs.presets.swap(index, new_index as usize);
        self.schedule_save_prefs()
    }

    fn apply_preset(&mut self, preset: &Preset) {
        self.font_size = preset.font_size;
        let enable_paths = preset
            .enable_paths
            .iter()
            .map(String::as_str)
            .collect::<HashSet<&str>>();
        for item in self.font_list_item_map.values_mut() {
            item.enabled = enable_paths.contains(item.font_entry.filepath.as_str());
        }
    }

    fn selected_preset_mut(&mut self) -> Option<&mut Preset> {
        let selected_name = self.prefs_selected_name.as_ref()?;
        self.prefs.as_mut().and_then(|prefs| {
            prefs
                .presets
                .iter_mut()
                .find(|preset| &preset.name == selected_name)
        })
    }

    fn create_font_entries(&self) -> Vec<FontEntry> {
        self.font_list_item_map
            .values()
            .filter(|item| item.enabled)
            .map(|item| item.font_entry.clone())
            .collect()
    }
}

#[derive(Clone, Debug)]
struct EditingPreset {
    original_name: String,
    new_name: String,
}

fn send_xmessage(msg: XMessage) -> Task<SettingsViewCommand> {
    Task::done(SettingsViewCommand::SendXMessage(msg))
}

fn settings_view_style(theme: &Theme) -> container::Style {
    let mut bg = theme.palette().background;
    bg.a = 0.9;
    background(bg)
}

fn next_preset_name(presets: &[Preset]) -> String {
    let existing: HashSet<&str> = presets.iter().map(|p| p.name.as_str()).collect();
    for i in 1.. {
        let name = format!("Preset {i}");
        if !existing.contains(name.as_str()) {
            return name;
        }
    }
    unreachable!()
}

const PRESET_ACTION_BTN_SIZE: f32 = 28.0;

fn preset_action_btn<'a>(
    content: impl Into<Element<'a, SettingsViewCommand>>,
) -> button::Button<'a, SettingsViewCommand> {
    button(
        container(content)
            .center_x(PRESET_ACTION_BTN_SIZE)
            .center_y(PRESET_ACTION_BTN_SIZE),
    )
    .width(PRESET_ACTION_BTN_SIZE)
    .height(PRESET_ACTION_BTN_SIZE)
    .padding(0)
}

fn pencil_icon<'a>() -> Element<'a, SettingsViewCommand> {
    svg_icon("pencil-solid.svg")
}

fn trash_icon<'a>() -> Element<'a, SettingsViewCommand> {
    svg_icon("trash-solid.svg")
}

fn svg_icon<'a>(name: &str) -> Element<'a, SettingsViewCommand> {
    let icon = Asset::get(name).unwrap_or_else(|| panic!("failed to load {name}"));
    container(
        svg(svg::Handle::from_memory(icon.data))
            .width(14.0)
            .height(14.0)
            .style(|_theme, _status| svg::Style {
                color: Some(Color::WHITE),
            }),
    )
    .padding(4.0)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn divider<'a>() -> Element<'a, SettingsViewCommand> {
    column![space().height(14), horizontal(1), space().height(14)].into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::font_list::FontListRepository;
    use iced::window;

    fn create_settings_view() -> SettingsView {
        let project_dirs =
            directories::ProjectDirs::from("com", "sukawasatoru", "fonts66-viewer").unwrap();
        let font_list_repo = Arc::new(FontListRepository::default());
        let prefs_repo = Arc::new(PreferencesRepository::new(&project_dirs));
        SettingsView::new(font_list_repo, prefs_repo)
    }

    fn setup_with_default_prefs() -> SettingsView {
        let mut view = create_settings_view();
        let prefs = view.prefs_repo.retrieve().unwrap();
        let _ = view.update(SettingsViewCommand::PrefsLoaded(prefs));
        view
    }

    fn setup_with_prefs(prefs: Preferences) -> SettingsView {
        let mut view = create_settings_view();
        let _ = view.update(SettingsViewCommand::PrefsLoaded(prefs));
        view
    }

    fn two_presets() -> Preferences {
        Preferences {
            presets: vec![
                Preset {
                    name: "Preset 1".into(),
                    font_size: 24,
                    enable_paths: vec!["./arial.ttf".into()],
                },
                Preset {
                    name: "Preset 2".into(),
                    font_size: 48,
                    enable_paths: vec!["./times.ttf".into()],
                },
            ],
        }
    }

    fn two_presets_no_paths() -> Preferences {
        Preferences {
            presets: vec![
                Preset {
                    name: "Preset 1".into(),
                    font_size: 24,
                    enable_paths: vec![],
                },
                Preset {
                    name: "Preset 2".into(),
                    font_size: 48,
                    enable_paths: vec![],
                },
            ],
        }
    }

    #[test]
    fn prefs_loaded_enables_only_preset_fonts() {
        let mut view = create_settings_view();

        // Before PrefsLoaded, all fonts are enabled.
        assert!(view.font_list_item_map.values().all(|item| item.enabled));

        let prefs = view.prefs_repo.retrieve().unwrap();
        let _ = view.update(SettingsViewCommand::PrefsLoaded(prefs));

        // After PrefsLoaded, only "./arial.ttf" (in fake preset) should be enabled.
        assert!(view.font_list_item_map.get("./arial.ttf").unwrap().enabled);
        assert!(!view.font_list_item_map.get("./times.ttf").unwrap().enabled);

        let entries = view.create_font_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].filepath, "./arial.ttf");
    }

    #[test]
    fn font_size_updated_sets_dirty_and_updates_preset() {
        let mut view = setup_with_default_prefs();
        assert_eq!(view.save_prefs_version, 0);
        assert_eq!(view.font_size, 24);

        let _ = view.update(SettingsViewCommand::FontSizeUpdated(48));

        assert!(view.save_prefs_version > 0);
        assert_eq!(view.font_size, 48);
        assert_eq!(view.prefs.as_ref().unwrap().presets[0].font_size, 48);
    }

    #[test]
    fn save_prefs_requested_clears_dirty_when_dirty() {
        let mut view = setup_with_default_prefs();
        let _ = view.update(SettingsViewCommand::FontSizeUpdated(48));
        let version = view.save_prefs_version;
        assert!(version > 0);

        let _ = view.update(SettingsViewCommand::SavePrefsRequested(version));
        assert_eq!(view.save_prefs_version, 0);

        // prefs in view should have updated font_size
        assert_eq!(view.prefs.as_ref().unwrap().presets[0].font_size, 48);
    }

    #[test]
    fn close_requested_saves_when_dirty() {
        let mut view = setup_with_default_prefs();
        let _ = view.update(SettingsViewCommand::FontSizeUpdated(48));
        assert!(view.save_prefs_version > 0);

        let _ = view.update(SettingsViewCommand::XMessage(XMessage::CloseRequested(
            window::Id::unique(),
        )));
        assert_eq!(view.save_prefs_version, 0);

        let saved = view.prefs_repo.retrieve().unwrap();
        assert_eq!(saved.presets[0].font_size, 48);
    }

    #[test]
    fn close_requested_skips_save_when_not_dirty() {
        let mut view = setup_with_default_prefs();
        assert_eq!(view.save_prefs_version, 0);
        let _ = view.update(SettingsViewCommand::XMessage(XMessage::CloseRequested(
            window::Id::unique(),
        )));
        assert_eq!(view.save_prefs_version, 0);

        let saved = view.prefs_repo.retrieve().unwrap();
        assert_eq!(saved.presets[0].font_size, 24);
    }

    #[test]
    fn save_prefs_requested_skips_when_not_dirty() {
        let mut view = setup_with_default_prefs();
        assert_eq!(view.save_prefs_version, 0);
        let _ = view.update(SettingsViewCommand::SavePrefsRequested(0));
        assert_eq!(view.save_prefs_version, 0);
    }

    #[test]
    fn preset_selected_switches_preset() {
        let mut view = setup_with_prefs(two_presets());

        assert_eq!(view.prefs_selected_name.as_deref(), Some("Preset 1"));
        assert_eq!(view.font_size, 24);
        assert!(view.font_list_item_map.get("./arial.ttf").unwrap().enabled);
        assert!(!view.font_list_item_map.get("./times.ttf").unwrap().enabled);

        let _ = view.update(SettingsViewCommand::PresetSelected("Preset 2".into()));

        assert_eq!(view.prefs_selected_name.as_deref(), Some("Preset 2"));
        assert_eq!(view.font_size, 48);
        assert!(!view.font_list_item_map.get("./arial.ttf").unwrap().enabled);
        assert!(view.font_list_item_map.get("./times.ttf").unwrap().enabled);
        assert!(view.save_prefs_version > 0);
    }

    #[test]
    fn preset_add_creates_new_preset_with_all_fonts_enabled() {
        let mut view = setup_with_default_prefs();
        assert_eq!(view.prefs.as_ref().unwrap().presets.len(), 1);

        let _ = view.update(SettingsViewCommand::PresetAddClicked);

        let prefs = view.prefs.as_ref().unwrap();
        assert_eq!(prefs.presets.len(), 2);
        assert_eq!(prefs.presets[1].name, "Preset 2");
        assert_eq!(prefs.presets[1].font_size, DEFAULT_SAMPLE_FONT_SIZE);
        assert_eq!(view.prefs_selected_name.as_deref(), Some("Preset 2"));
        assert_eq!(view.font_size, DEFAULT_SAMPLE_FONT_SIZE);
        assert!(view.font_list_item_map.values().all(|item| item.enabled));
        assert!(view.save_prefs_version > 0);
    }

    #[test]
    fn preset_move_up_swaps_with_previous() {
        let mut view = setup_with_prefs(two_presets_no_paths());

        let _ = view.update(SettingsViewCommand::PresetMoveUp("Preset 2".into()));

        let names: Vec<&str> = view
            .prefs
            .as_ref()
            .unwrap()
            .presets
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert_eq!(names, vec!["Preset 2", "Preset 1"]);
        assert!(view.save_prefs_version > 0);
    }

    #[test]
    fn preset_move_up_noop_when_first() {
        let mut view = setup_with_default_prefs();
        let _ = view.update(SettingsViewCommand::PresetMoveUp("Preset 1".into()));
        assert_eq!(view.save_prefs_version, 0);
    }

    #[test]
    fn preset_move_down_swaps_with_next() {
        let mut view = setup_with_prefs(two_presets_no_paths());

        let _ = view.update(SettingsViewCommand::PresetMoveDown("Preset 1".into()));

        let names: Vec<&str> = view
            .prefs
            .as_ref()
            .unwrap()
            .presets
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert_eq!(names, vec!["Preset 2", "Preset 1"]);
        assert!(view.save_prefs_version > 0);
    }

    #[test]
    fn preset_move_down_noop_when_last() {
        let mut view = setup_with_default_prefs();
        let _ = view.update(SettingsViewCommand::PresetMoveDown("Preset 1".into()));
        assert_eq!(view.save_prefs_version, 0);
    }

    #[test]
    fn preset_add_skips_existing_names() {
        let mut view = setup_with_prefs(two_presets());
        let _ = view.update(SettingsViewCommand::PresetAddClicked);

        let prefs = view.prefs.as_ref().unwrap();
        assert_eq!(prefs.presets.len(), 3);
        assert_eq!(prefs.presets[2].name, "Preset 3");
    }

    #[test]
    fn preset_delete_removes_and_selects_next() {
        let mut view = setup_with_prefs(two_presets());
        assert_eq!(view.prefs_selected_name.as_deref(), Some("Preset 1"));

        let _ = view.update(SettingsViewCommand::PresetDeleteClicked("Preset 1".into()));

        let prefs = view.prefs.as_ref().unwrap();
        assert_eq!(prefs.presets.len(), 1);
        assert_eq!(prefs.presets[0].name, "Preset 2");
        assert_eq!(view.prefs_selected_name.as_deref(), Some("Preset 2"));
        assert_eq!(view.font_size, 48);
        assert!(view.font_list_item_map.get("./times.ttf").unwrap().enabled);
        assert!(view.save_prefs_version > 0);
    }

    #[test]
    fn preset_delete_non_selected_keeps_selection() {
        let mut view = setup_with_prefs(two_presets());

        let _ = view.update(SettingsViewCommand::PresetDeleteClicked("Preset 2".into()));

        let prefs = view.prefs.as_ref().unwrap();
        assert_eq!(prefs.presets.len(), 1);
        assert_eq!(view.prefs_selected_name.as_deref(), Some("Preset 1"));
        assert_eq!(view.font_size, 24);
        assert!(view.save_prefs_version > 0);
    }

    #[test]
    fn preset_delete_noop_when_single_preset() {
        let mut view = setup_with_default_prefs();
        assert_eq!(view.prefs.as_ref().unwrap().presets.len(), 1);
        let _ = view.update(SettingsViewCommand::PresetDeleteClicked("Preset 1".into()));

        assert_eq!(view.prefs.as_ref().unwrap().presets.len(), 1);
        assert_eq!(view.save_prefs_version, 0);
    }

    #[test]
    fn preset_delete_last_selects_previous() {
        let mut view = setup_with_prefs(two_presets());
        let _ = view.update(SettingsViewCommand::PresetSelected("Preset 2".into()));
        view.save_prefs_version = 0;

        let _ = view.update(SettingsViewCommand::PresetDeleteClicked("Preset 2".into()));

        let prefs = view.prefs.as_ref().unwrap();
        assert_eq!(prefs.presets.len(), 1);
        assert_eq!(view.prefs_selected_name.as_deref(), Some("Preset 1"));
        assert_eq!(view.font_size, 24);
    }

    #[test]
    fn preset_rename_changes_name() {
        let mut view = setup_with_default_prefs();

        let _ = view.update(SettingsViewCommand::PresetRenameStarted("Preset 1".into()));
        assert!(view.editing_preset.is_some());

        let _ = view.update(SettingsViewCommand::PresetRenameChanged("My Preset".into()));
        assert_eq!(view.editing_preset.as_ref().unwrap().new_name, "My Preset");

        let _ = view.update(SettingsViewCommand::PresetRenameConfirmed);

        assert!(view.editing_preset.is_none());
        assert_eq!(view.prefs.as_ref().unwrap().presets[0].name, "My Preset");
        assert_eq!(view.prefs_selected_name.as_deref(), Some("My Preset"));
        assert!(view.save_prefs_version > 0);
    }

    #[test]
    fn preset_rename_updates_selected_name() {
        let mut view = setup_with_prefs(two_presets_no_paths());
        let _ = view.update(SettingsViewCommand::PresetSelected("Preset 2".into()));
        view.save_prefs_version = 0;

        let _ = view.update(SettingsViewCommand::PresetRenameStarted("Preset 2".into()));
        let _ = view.update(SettingsViewCommand::PresetRenameChanged("Renamed".into()));
        let _ = view.update(SettingsViewCommand::PresetRenameConfirmed);

        assert_eq!(view.prefs_selected_name.as_deref(), Some("Renamed"));
        assert_eq!(view.prefs.as_ref().unwrap().presets[1].name, "Renamed");
    }

    #[test]
    fn preset_rename_does_not_update_unselected_name() {
        let mut view = setup_with_prefs(two_presets_no_paths());

        let _ = view.update(SettingsViewCommand::PresetRenameStarted("Preset 2".into()));
        let _ = view.update(SettingsViewCommand::PresetRenameChanged("Renamed".into()));
        let _ = view.update(SettingsViewCommand::PresetRenameConfirmed);

        assert_eq!(view.prefs_selected_name.as_deref(), Some("Preset 1"));
        assert_eq!(view.prefs.as_ref().unwrap().presets[1].name, "Renamed");
    }

    #[test]
    fn preset_rename_noop_when_empty() {
        let mut view = setup_with_default_prefs();

        let _ = view.update(SettingsViewCommand::PresetRenameStarted("Preset 1".into()));
        let _ = view.update(SettingsViewCommand::PresetRenameChanged("  ".into()));
        let _ = view.update(SettingsViewCommand::PresetRenameConfirmed);

        assert_eq!(view.prefs.as_ref().unwrap().presets[0].name, "Preset 1");
        assert_eq!(view.save_prefs_version, 0);
    }

    #[test]
    fn preset_rename_noop_when_duplicate() {
        let mut view = setup_with_prefs(two_presets_no_paths());

        let _ = view.update(SettingsViewCommand::PresetRenameStarted("Preset 1".into()));
        let _ = view.update(SettingsViewCommand::PresetRenameChanged("Preset 2".into()));
        let _ = view.update(SettingsViewCommand::PresetRenameConfirmed);

        assert_eq!(view.prefs.as_ref().unwrap().presets[0].name, "Preset 1");
        assert_eq!(view.save_prefs_version, 0);
    }

    #[test]
    fn preset_rename_noop_when_unchanged() {
        let mut view = setup_with_default_prefs();

        let _ = view.update(SettingsViewCommand::PresetRenameStarted("Preset 1".into()));
        let _ = view.update(SettingsViewCommand::PresetRenameConfirmed);

        assert_eq!(view.prefs.as_ref().unwrap().presets[0].name, "Preset 1");
        assert_eq!(view.save_prefs_version, 0);
    }

    #[test]
    fn preset_selected_noop_when_already_selected() {
        let mut view = setup_with_default_prefs();
        assert_eq!(view.save_prefs_version, 0);
        let _ = view.update(SettingsViewCommand::PresetSelected("Preset 1".into()));
        assert_eq!(view.save_prefs_version, 0);
    }

    #[test]
    fn save_prefs_requested_skips_stale_version() {
        let mut view = setup_with_default_prefs();
        let _ = view.update(SettingsViewCommand::FontSizeUpdated(48));
        let stale_version = view.save_prefs_version;

        // Another update bumps the version.
        let _ = view.update(SettingsViewCommand::FontSizeUpdated(64));
        let latest_version = view.save_prefs_version;
        assert_ne!(stale_version, latest_version);

        // Stale version is ignored.
        let _ = view.update(SettingsViewCommand::SavePrefsRequested(stale_version));
        assert_eq!(view.save_prefs_version, latest_version);

        // Latest version triggers save and resets version.
        let _ = view.update(SettingsViewCommand::SavePrefsRequested(latest_version));
        assert_eq!(view.save_prefs_version, 0);
    }
}
