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
use crate::asset::Asset;
use iced::widget::svg::Handle as SvgHandle;
use iced::widget::{button, svg};
use iced::{Border, Element, Theme};

pub fn settings_button_outline<'a, Message: Clone + 'a>(on_press: Message) -> Element<'a, Message> {
    settings_button("paint-brush-outline.svg", on_press)
}

pub fn settings_button_solid<'a, Message: Clone + 'a>(on_press: Message) -> Element<'a, Message> {
    settings_button("paint-brush-solid.svg", on_press)
}

fn settings_button<'a, Message: Clone + 'a>(
    icon_path: &str,
    on_press: Message,
) -> Element<'a, Message> {
    let icon = Asset::get(icon_path).expect("failed to load svg");
    button(svg(SvgHandle::from_memory(icon.data)).width(24).height(24))
        .height(28)
        .on_press(on_press)
        .style(button_style)
        .into()
}

fn button_style(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::subtle(theme, status);
    style.border = Border {
        radius: 999.into(),
        ..style.border
    };
    style
}
