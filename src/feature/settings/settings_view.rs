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
use iced::widget::rule::horizontal;
use iced::widget::{column, container, row, scrollable, space, text_editor};
use iced::{Alignment, Element, Length, Subscription, Task, Theme, padding};
use iced_aw::number_input;
use indexmap::IndexMap;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum SettingsViewCommand {
    FontListItemChanged(FontEntry, bool),
    FontSizeUpdated(u32),
    PrefsLoaded(Preferences),
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
                    Task::done(SettingsViewCommand::SendXMessage(XMessage::FontEntries(
                        self.create_font_entries(),
                    ))),
                    self.schedule_save_prefs(),
                ])
            }
            SettingsViewCommand::FontSizeUpdated(size) => {
                self.font_size = size;

                if let Some(preset) = self.selected_preset_mut() {
                    preset.font_size = size;
                }

                Task::batch([
                    Task::done(SettingsViewCommand::SendXMessage(XMessage::FontSize(size))),
                    self.schedule_save_prefs(),
                ])
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

                if let Some(selected_name) = &self.prefs_selected_name {
                    for preset in &prefs.presets {
                        if selected_name == &preset.name {
                            self.font_size = preset.font_size;
                            let enable_paths = preset
                                .enable_paths
                                .iter()
                                .map(String::as_str)
                                .collect::<HashSet<_>>();
                            for item in self.font_list_item_map.values_mut() {
                                item.enabled =
                                    enable_paths.contains(item.font_entry.filepath.as_str());
                            }
                            break;
                        }
                    }
                }

                self.prefs = Some(prefs);

                Task::batch([
                    Task::done(SettingsViewCommand::SendXMessage(XMessage::FontSize(
                        self.font_size,
                    ))),
                    Task::done(SettingsViewCommand::SendXMessage(XMessage::FontEntries(
                        self.create_font_entries(),
                    ))),
                ])
            }
            SettingsViewCommand::SavePrefsRequested(version) => {
                if version != self.save_prefs_version {
                    return Task::none();
                }
                // Reset to 0 to indicate no pending unsaved changes.
                self.save_prefs_version = 0;

                let prefs_repo = self.prefs_repo.clone();
                self.prefs
                    .as_ref()
                    .map(|prefs| {
                        let prefs = prefs.clone();
                        Task::perform(
                            async move {
                                match prefs_repo.save(prefs) {
                                    Ok(_) => info!("saved preferences"),
                                    Err(e) => warn!(?e, "failed to save preferences"),
                                };
                            },
                            |_| SettingsViewCommand::Sink,
                        )
                    })
                    .unwrap_or(Task::none())
            }
            SettingsViewCommand::SettingsButtonClicked => {
                Task::done(SettingsViewCommand::SendXMessage(XMessage::SettingsClose))
            }
            SettingsViewCommand::TextEditorAction(action) => {
                let need_update = matches!(&action, text_editor::Action::Edit(_));

                self.custom_text_content.perform(action);

                if need_update {
                    Task::done(SettingsViewCommand::SendXMessage(XMessage::CustomText(
                        self.custom_text_content.text(),
                    )))
                } else {
                    Task::none()
                }
            }
            // Propagate to App layer via Task so it can be converted to AppCommand::XMessage.
            SettingsViewCommand::SendXMessage(data) => {
                Task::done(SettingsViewCommand::SendXMessage(data))
            }
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

        let mut content_inner = column![
            container(
                text_editor(&self.custom_text_content)
                    .placeholder("Custom text...")
                    .on_action(SettingsViewCommand::TextEditorAction),
            )
            .height(73),
            divider(),
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
            divider(),
        ];

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

fn settings_view_style(theme: &Theme) -> container::Style {
    let mut bg = theme.palette().background;
    bg.a = 0.9;
    background(bg)
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

    #[test]
    fn prefs_loaded_enables_only_preset_fonts() {
        let mut view = create_settings_view();

        // Before PrefsLoaded, all fonts are enabled.
        assert!(view.font_list_item_map.values().all(|item| item.enabled));

        let prefs = view.prefs_repo.retrieve().unwrap();
        let _ = view.update(SettingsViewCommand::PrefsLoaded(prefs));

        // After PrefsLoaded, only "./arial.ttf" (in fake preset) should be enabled.
        let arial = view.font_list_item_map.get("./arial.ttf").unwrap();
        assert!(arial.enabled);

        let times = view.font_list_item_map.get("./times.ttf").unwrap();
        assert!(!times.enabled);

        let entries = view.create_font_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].filepath, "./arial.ttf");
    }

    #[test]
    fn font_size_updated_sets_dirty_and_updates_preset() {
        let mut view = create_settings_view();
        let prefs = view.prefs_repo.retrieve().unwrap();
        let _ = view.update(SettingsViewCommand::PrefsLoaded(prefs));

        assert_eq!(view.save_prefs_version, 0);
        assert_eq!(view.font_size, 24);

        let _ = view.update(SettingsViewCommand::FontSizeUpdated(48));

        assert!(view.save_prefs_version > 0);
        assert_eq!(view.font_size, 48);
        assert_eq!(view.prefs.as_ref().unwrap().presets[0].font_size, 48);
    }

    #[test]
    fn save_prefs_requested_clears_dirty_when_dirty() {
        let mut view = create_settings_view();
        let prefs = view.prefs_repo.retrieve().unwrap();
        let _ = view.update(SettingsViewCommand::PrefsLoaded(prefs));

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
        let mut view = create_settings_view();
        let prefs = view.prefs_repo.retrieve().unwrap();
        let _ = view.update(SettingsViewCommand::PrefsLoaded(prefs));

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
        let mut view = create_settings_view();
        let prefs = view.prefs_repo.retrieve().unwrap();
        let _ = view.update(SettingsViewCommand::PrefsLoaded(prefs));

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
        let mut view = create_settings_view();
        let prefs = view.prefs_repo.retrieve().unwrap();
        let _ = view.update(SettingsViewCommand::PrefsLoaded(prefs));

        assert_eq!(view.save_prefs_version, 0);
        let _ = view.update(SettingsViewCommand::SavePrefsRequested(0));
        assert_eq!(view.save_prefs_version, 0);
    }

    #[test]
    fn save_prefs_requested_skips_stale_version() {
        let mut view = create_settings_view();
        let prefs = view.prefs_repo.retrieve().unwrap();
        let _ = view.update(SettingsViewCommand::PrefsLoaded(prefs));

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
