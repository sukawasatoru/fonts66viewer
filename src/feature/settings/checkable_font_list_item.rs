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
use crate::feature::settings::SettingsViewCommand;
use crate::feature::settings::font_list_item::FontListItem;
use iced::Element;
use iced::widget::{checkbox, row};

pub fn checkable_font_list_item(
    font_list_item: &'_ FontListItem,
) -> Element<'_, SettingsViewCommand> {
    let label = font_list_item
        .font_entry
        .display_name
        .as_deref()
        .unwrap_or(font_list_item.font_entry.font_name);
    let font_entry = font_list_item.font_entry.clone();
    row![
        checkbox(font_list_item.enabled)
            .label(label)
            .on_toggle(move |enabled| {
                SettingsViewCommand::FontListItemChanged(font_entry.clone(), enabled)
            }),
    ]
    .into()
}
