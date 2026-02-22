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
use crate::feature::settings::checkable_font_list_item::checkable_font_list_item;
use crate::feature::settings::font_list_item::FontListItem;
use crate::model::{
    DEFAULT_SAMPLE_FONT_SIZE, FontEntry, TOOLBAR_HEIGHT, WINDOW_BODY_MARGIN, XMessage,
};
use crate::widget::settings_button_solid;
use iced::widget::container::background;
use iced::widget::rule::horizontal;
use iced::widget::{column, container, row, scrollable, space, text_editor};
use iced::{Alignment, Element, Length, Subscription, Task, Theme, padding};
use iced_aw::number_input;
use indexmap::IndexMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum SettingsViewCommand {
    FontListItemChanged(FontEntry, bool),
    FontSizeUpdated(u32),
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
}
impl SettingsView {
    pub fn new(font_list_repo: Arc<FontListRepository>) -> Self {
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
        }
    }

    pub fn update(&mut self, command: SettingsViewCommand) -> Task<SettingsViewCommand> {
        match command {
            SettingsViewCommand::FontListItemChanged(font_entry, enabled) => {
                self.font_list_item_map
                    .get_mut(&font_entry.filepath)
                    .expect("SettingsView should have entry")
                    .enabled = enabled;
                Task::done(SettingsViewCommand::SendXMessage(XMessage::FontEntries(
                    self.create_font_entries(),
                )))
            }
            SettingsViewCommand::FontSizeUpdated(size) => {
                self.font_size = size;
                Task::done(SettingsViewCommand::SendXMessage(XMessage::FontSize(size)))
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
            SettingsViewCommand::XMessage(message) => {
                if matches!(message, XMessage::Init) {
                    Task::done(SettingsViewCommand::SendXMessage(XMessage::FontEntries(
                        self.create_font_entries(),
                    )))
                } else {
                    Task::none()
                }
            }
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

    fn create_font_entries(&self) -> Vec<FontEntry> {
        self.font_list_item_map
            .values()
            .filter_map(|list_item| {
                if list_item.enabled {
                    Some(list_item.font_entry.clone())
                } else {
                    None
                }
            })
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
