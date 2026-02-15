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
use crate::model::FontEntry;
use iced::widget::rule::horizontal;
use iced::widget::{column, scrollable, text};
use iced::{Element, Font, Length, Subscription, Task};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum MainViewCommand {
    //
}

pub struct MainView {
    font_entries: Vec<FontEntry>,
}

impl MainView {
    pub fn new(font_list_repo: Arc<FontListRepository>) -> Self {
        let font_entries = font_list_repo.find_all();
        Self { font_entries }
    }

    pub fn update(&mut self, _command: MainViewCommand) -> Task<MainViewCommand> {
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<MainViewCommand> {
        Subscription::none()
    }

    pub fn view(&self) -> Element<'_, MainViewCommand> {
        let mut content = column![];

        let mut first = true;
        for entry in &self.font_entries {
            if !first {
                content = content.push(horizontal(1));
            } else {
                first = false;
            }

            content = content.push(list_item(
                entry,
                "あのイーハトーヴォのすきとおった風、夏でも底に冷たさをもつ青いそら、うつくしい森で飾られたモリーオ市、郊外のぎらぎらひかる草の波。",
             ));
        }

        scrollable(content.width(Length::Fill)).into()
    }
}

fn list_item<'a>(font_entry: &'a FontEntry, message: &'a str) -> Element<'a, MainViewCommand> {
    let content = column![];

    let content = match font_entry.display_name {
        Some(ref data) => content.push(text(data)),
        None => content.push(font_entry.font_name),
    };

    content
        .push(text(font_entry.font_name))
        .push(
            text(message)
                .size(28)
                .font(Font::with_name(font_entry.font_name)),
        )
        .into()
}
