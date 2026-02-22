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
use crate::model::{DEFAULT_SAMPLE_FONT_SIZE, FontEntry, TOOLBAR_HEIGHT, XMessage};
use iced::widget::rule::horizontal;
use iced::widget::{column, scrollable, space, text};
use iced::{Element, Font, Length, Subscription, Task};

#[derive(Clone, Debug)]
pub enum MainViewCommand {
    SendXMessage(XMessage),
    XMessage(XMessage),
}

pub struct MainView {
    custom_text: String,
    font_size: u32,
    font_entries: Vec<FontEntry>,
}

impl MainView {
    pub fn new() -> Self {
        Self {
            custom_text: "".to_owned(),
            font_size: DEFAULT_SAMPLE_FONT_SIZE,
            font_entries: vec![],
        }
    }

    pub fn update(&mut self, command: MainViewCommand) -> Task<MainViewCommand> {
        match command {
            // Propagate to App layer via Task so it can be converted to AppCommand::XMessage.
            MainViewCommand::SendXMessage(data) => Task::done(MainViewCommand::SendXMessage(data)),
            MainViewCommand::XMessage(message) => match message {
                XMessage::CustomText(value) => {
                    self.custom_text = value;
                    Task::none()
                }
                XMessage::FontEntries(entries) => {
                    self.font_entries = entries;
                    Task::none()
                }
                XMessage::FontSize(size) => {
                    self.font_size = size;
                    Task::none()
                }
                _ => Task::none(),
            },
        }
    }

    pub fn subscription(&self) -> Subscription<MainViewCommand> {
        Subscription::none()
    }

    pub fn view(&self) -> Element<'_, MainViewCommand> {
        let mut content = column![space().height(TOOLBAR_HEIGHT)];

        let mut first = true;
        for entry in &self.font_entries {
            if !first {
                content = content.push(horizontal(1));
            } else {
                first = false;
            }

            content = content.push(list_item(
                entry,
                self.font_size,
                if self.custom_text.is_empty() {
                    "あのイーハトーヴォのすきとおった風、夏でも底に冷たさをもつ青いそら、うつくしい森で飾られたモリーオ市、郊外のぎらぎらひかる草の波。"
                } else {
                    &self.custom_text
                },
             ));
        }

        scrollable(content.width(Length::Fill)).into()
    }
}

impl Default for MainView {
    fn default() -> Self {
        Self::new()
    }
}

fn list_item<'a>(
    font_entry: &'a FontEntry,
    font_size: u32,
    message: &'a str,
) -> Element<'a, MainViewCommand> {
    let content = column![];

    let content = match font_entry.display_name {
        Some(ref data) => content.push(text(data)),
        None => content.push(font_entry.font_name),
    };

    content
        .push(text(font_entry.font_name))
        .push(
            text(message)
                .size(font_size)
                .font(Font::with_name(font_entry.font_name)),
        )
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::font_list::FontListRepository;
    use iced::Theme;

    #[test]
    #[ignore = "snapshot testing"]
    fn test_simulator() {
        let font_entries = FontListRepository::default().find_all();
        let mut main_view = MainView::new();
        let _ = main_view.update(MainViewCommand::XMessage(XMessage::FontEntries(
            font_entries,
        )));

        let mut simulator = iced_test::simulator(main_view.view());
        let snapshot = simulator.snapshot(&Theme::Light).unwrap();
        assert!(
            snapshot
                .matches_hash(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/snapshots/main_view_launch"
                ),)
                .unwrap(),
            "snapshot should match",
        )
    }
}
