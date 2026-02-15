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
use crate::feature::main::{MainView, MainViewCommand};
use iced::{Element, Subscription, Task, Theme};
use std::sync::Arc;

pub fn boot() -> (AppState, Task<AppCommand>) {
    let font_list_repo = Arc::new(FontListRepository::new());

    let state = AppState {
        view_main: MainView::new(font_list_repo),
        theme: Theme::Light,
    };

    (state, Task::none())
}

#[derive(Clone, Debug)]
pub enum AppCommand {
    MainViewCommand(MainViewCommand),
}

pub struct AppState {
    theme: Theme,
    view_main: MainView,
}

impl AppState {
    pub fn title(&self) -> String {
        "Fonts66 Viewer".to_string()
    }

    pub fn update(&mut self, message: AppCommand) -> Task<AppCommand> {
        match message {
            AppCommand::MainViewCommand(command) => self
                .view_main
                .update(command)
                .map(AppCommand::MainViewCommand),
        }
    }

    pub fn view(&self) -> Element<'_, AppCommand> {
        self.view_main.view().map(AppCommand::MainViewCommand)
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn subscription(&self) -> Subscription<AppCommand> {
        self.view_main
            .subscription()
            .map(AppCommand::MainViewCommand)
    }
}
