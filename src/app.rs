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
use crate::model::XMessage;
use iced::{Application, Element, Program, Subscription, Task, Theme};
use std::sync::Arc;

pub fn application() -> Application<impl Program<State = AppState>> {
    iced::application(boot, AppState::update, AppState::view)
        .title(AppState::title)
        .subscription(AppState::subscription)
        .theme(AppState::theme)
}

fn boot() -> (AppState, Task<AppCommand>) {
    let font_list_repo = Arc::new(FontListRepository::default());

    let state = AppState {
        view_main: MainView::new(font_list_repo),
        theme: Theme::Light,
    };

    (state, Task::none())
}

#[derive(Clone, Debug)]
enum AppCommand {
    MainViewCommand(MainViewCommand),
    XMessage(XMessage),
}

pub struct AppState {
    theme: Theme,
    view_main: MainView,
}

impl AppState {
    fn title(&self) -> String {
        "Fonts66 Viewer".to_string()
    }

    fn update(&mut self, message: AppCommand) -> Task<AppCommand> {
        match message {
            AppCommand::MainViewCommand(command) => {
                self.view_main.update(command).map(|command| match command {
                    MainViewCommand::SendXMessage(message) => AppCommand::XMessage(message),
                    _ => AppCommand::MainViewCommand(command),
                })
            }
            AppCommand::XMessage(message) => match message {
                XMessage::Exit => iced::exit(),
                #[allow(unreachable_patterns)]
                _ => self
                    .view_main
                    .update(MainViewCommand::XMessage(message))
                    .map(AppCommand::MainViewCommand),
            },
        }
    }

    fn view(&self) -> Element<'_, AppCommand> {
        self.view_main.view().map(AppCommand::MainViewCommand)
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<AppCommand> {
        self.view_main
            .subscription()
            .map(AppCommand::MainViewCommand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Preset;

    #[test]
    #[ignore = "E2E testing"]
    fn screenshot() -> Result<(), iced_test::Error> {
        let app = application().presets([Preset::new("Empty", || {
            let state = AppState {
                theme: Theme::Light,
                view_main: MainView::new(Arc::new(FontListRepository::default())),
            };

            (state, Task::none())
        })]);

        // .ice: https://github.com/iced-rs/iced/blob/0.14.0/test/src/ice.rs
        //       https://github.com/iced-rs/iced/blob/0.14.0/test/src/instruction.rs
        iced_test::run(app, format!("{}/tests", env!("CARGO_MANIFEST_DIR")))
    }
}
