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
use crate::feature::settings::{SettingsView, SettingsViewCommand};
use crate::feature::toolbar::{Toolbar, ToolbarCommand};
use crate::model::XMessage;
use iced::keyboard;
use iced::widget::{opaque, right, space, stack};
use iced::{Application, Element, Program, Subscription, Task, Theme};
use std::sync::Arc;

pub fn application() -> Application<impl Program<State = AppState, Message = AppCommand>> {
    iced::application(boot, AppState::update, AppState::view)
        .title(AppState::title)
        .subscription(AppState::subscription)
        .theme(AppState::theme)
}

fn boot() -> (AppState, Task<AppCommand>) {
    let font_list_repo = Arc::new(FontListRepository::default());

    let state = AppState {
        expand_settings: false,
        view_main: MainView::new(),
        view_settings: SettingsView::new(font_list_repo),
        view_toolbar: Toolbar::new(),
        theme: Theme::Light,
    };

    (state, Task::done(AppCommand::XMessage(XMessage::Init)))
}

#[derive(Clone, Debug)]
pub enum AppCommand {
    Esc,
    MainViewCommand(MainViewCommand),
    SettingsViewCommand(SettingsViewCommand),
    ToolbarCommand(ToolbarCommand),
    XMessage(XMessage),
}

pub struct AppState {
    expand_settings: bool,
    theme: Theme,
    view_main: MainView,
    view_settings: SettingsView,
    view_toolbar: Toolbar,
}

impl AppState {
    fn title(&self) -> String {
        "Fonts66 Viewer".to_string()
    }

    fn update(&mut self, message: AppCommand) -> Task<AppCommand> {
        match message {
            AppCommand::Esc => {
                if self.expand_settings {
                    self.expand_settings = false;
                    Task::none()
                } else {
                    iced::exit()
                }
            }
            AppCommand::MainViewCommand(command) => {
                self.view_main.update(command).map(|command| match command {
                    MainViewCommand::SendXMessage(message) => AppCommand::XMessage(message),
                    _ => AppCommand::MainViewCommand(command),
                })
            }
            AppCommand::SettingsViewCommand(command) => {
                self.view_settings
                    .update(command)
                    .map(|command| match command {
                        SettingsViewCommand::SendXMessage(message) => AppCommand::XMessage(message),
                        _ => AppCommand::SettingsViewCommand(command),
                    })
            }
            AppCommand::ToolbarCommand(command) => {
                self.view_toolbar
                    .update(command)
                    .map(|command| match command {
                        ToolbarCommand::SendXMessage(message) => AppCommand::XMessage(message),
                        _ => AppCommand::ToolbarCommand(command),
                    })
            }
            AppCommand::XMessage(message) => match message {
                XMessage::SettingsClose => {
                    self.expand_settings = false;
                    Task::none()
                }
                XMessage::SettingsOpen => {
                    self.expand_settings = true;
                    Task::none()
                }
                _ => Task::batch([
                    self.view_main
                        .update(MainViewCommand::XMessage(message.clone()))
                        .map(AppCommand::MainViewCommand),
                    self.view_settings
                        .update(SettingsViewCommand::XMessage(message.clone()))
                        .map(AppCommand::SettingsViewCommand),
                    self.view_toolbar
                        .update(ToolbarCommand::XMessage(message))
                        .map(AppCommand::ToolbarCommand),
                ]),
            },
        }
    }

    fn view(&self) -> Element<'_, AppCommand> {
        stack([
            self.view_main.view().map(AppCommand::MainViewCommand),
            opaque(self.view_toolbar.view().map(AppCommand::ToolbarCommand)),
            if self.expand_settings {
                right(opaque(
                    self.view_settings
                        .view()
                        .map(AppCommand::SettingsViewCommand),
                ))
                .into()
            } else {
                space().into()
            },
        ])
        .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<AppCommand> {
        Subscription::batch([
            iced::event::listen_with(|event, _status, _id| match event {
                iced::Event::Keyboard(keyboard::Event::KeyReleased {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }) => Some(AppCommand::Esc),
                _ => None,
            }),
            self.view_main
                .subscription()
                .map(AppCommand::MainViewCommand),
            self.view_settings
                .subscription()
                .map(AppCommand::SettingsViewCommand),
            self.view_toolbar
                .subscription()
                .map(AppCommand::ToolbarCommand),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Preset;

    #[test]
    #[ignore = "E2E testing"]
    fn screenshot() -> Result<(), iced_test::Error> {
        let app = application().presets([Preset::new("Empty", boot)]);

        // .ice: https://github.com/iced-rs/iced/blob/0.14.0/test/src/ice.rs
        //       https://github.com/iced-rs/iced/blob/0.14.0/test/src/instruction.rs
        iced_test::run(app, format!("{}/tests", env!("CARGO_MANIFEST_DIR")))
    }
}
