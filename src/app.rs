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
use crate::feature::main::{MainView, MainViewCommand};
use crate::feature::settings::{SettingsView, SettingsViewCommand};
use crate::feature::toolbar::{Toolbar, ToolbarCommand};
use crate::model::XMessage;
use iced::keyboard;
use iced::widget::{opaque, right, space, stack};
use iced::window;
use iced::{Application, Element, Program, Subscription, Task, Theme};
use std::sync::Arc;

pub fn application() -> Application<impl Program<State = AppState, Message = AppCommand>> {
    iced::application(boot, AppState::update, AppState::view)
        .title(AppState::title)
        .subscription(AppState::subscription)
        .theme(AppState::theme)
        // Disable default close behavior so that window::close_requests()
        // events reach the subscription, allowing each feature to save dirty
        // state before the window is actually closed.
        .exit_on_close_request(false)
}

fn boot() -> (AppState, Task<AppCommand>) {
    let project_dirs = directories::ProjectDirs::from("com", "sukawasatoru", "Fonts66 Viewer")
        .expect("no valid home directory");
    let font_list_repo = Arc::new(FontListRepository::default());
    let prefs_repo = Arc::new(PreferencesRepository::new(&project_dirs));

    let state = AppState {
        expand_settings: false,
        view_main: MainView::new(),
        view_settings: SettingsView::new(font_list_repo, prefs_repo),
        view_toolbar: Toolbar::new(),
        theme: Theme::Light,
    };

    (state, Task::done(AppCommand::XMessage(XMessage::Init)))
}

#[derive(Clone, Debug)]
pub enum AppCommand {
    Esc(window::Id),
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
            AppCommand::Esc(id) => {
                if self.expand_settings {
                    self.expand_settings = false;
                    Task::none()
                } else {
                    Task::done(AppCommand::XMessage(XMessage::CloseRequested(id)))
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
            AppCommand::XMessage(message) => {
                match &message {
                    XMessage::SettingsClose => {
                        self.expand_settings = false;
                    }
                    XMessage::SettingsOpen => {
                        self.expand_settings = true;
                    }
                    _ => {}
                }

                // After broadcasting CloseRequested to all features (so they
                // can synchronously save dirty preferences), close the window.
                let close_task = match &message {
                    XMessage::CloseRequested(id) => window::close(*id),
                    _ => Task::none(),
                };

                let tasks = self.broadcast_xmessage(message);

                Task::batch([tasks, close_task])
            }
        }
    }

    fn broadcast_xmessage(&mut self, message: XMessage) -> Task<AppCommand> {
        Task::batch([
            self.view_main
                .update(MainViewCommand::XMessage(message.clone()))
                .map(AppCommand::MainViewCommand),
            self.view_settings
                .update(SettingsViewCommand::XMessage(message.clone()))
                .map(AppCommand::SettingsViewCommand),
            self.view_toolbar
                .update(ToolbarCommand::XMessage(message))
                .map(AppCommand::ToolbarCommand),
        ])
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
            iced::event::listen_with(|event, _status, id| match event {
                // ESC key triggers CloseRequested via Task::done so that the
                // same shutdown flow (broadcast + window::close) is shared.
                iced::Event::Keyboard(keyboard::Event::KeyReleased {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }) => Some(AppCommand::Esc(id)),
                iced::Event::Window(window::Event::CloseRequested) => {
                    Some(AppCommand::XMessage(XMessage::CloseRequested(id)))
                }
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
