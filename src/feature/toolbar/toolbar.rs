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
use crate::model::{TOOLBAR_HEIGHT, WINDOW_BODY_MARGIN, XMessage};
use crate::widget::settings_button_outline;
use iced::widget::{container, row, space};
use iced::{Element, Length, Subscription, Task, Theme};

#[derive(Clone, Debug)]
pub enum ToolbarCommand {
    SendXMessage(XMessage),
    XMessage(XMessage),
}

#[derive(Default)]
pub struct Toolbar;

impl Toolbar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, command: ToolbarCommand) -> Task<ToolbarCommand> {
        match command {
            // Propagate to App layer via Task so it can be converted to AppCommand::XMessage.
            ToolbarCommand::SendXMessage(data) => Task::done(ToolbarCommand::SendXMessage(data)),
            ToolbarCommand::XMessage(_) => Task::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<ToolbarCommand> {
        Subscription::none()
    }

    pub fn view(&self) -> Element<'_, ToolbarCommand> {
        let content = row![
            space().width(Length::Fill),
            settings_button_outline(ToolbarCommand::SendXMessage(XMessage::SettingsOpen)),
            space().width(WINDOW_BODY_MARGIN),
        ];

        container(content.width(Length::Fill))
            .width(Length::Fill)
            .center_y(TOOLBAR_HEIGHT)
            .style(toolbar_style)
            .into()
    }
}

fn toolbar_style(theme: &Theme) -> container::Style {
    let mut bg = theme.palette().background;
    bg.a = 0.9;

    container::primary(theme).background(bg)
}
