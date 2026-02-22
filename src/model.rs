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

pub use font_entry::FontEntry;
pub use x_message::XMessage;

mod font_entry;
mod x_message;

pub const DEFAULT_SAMPLE_FONT_SIZE: u32 = 28;
pub const TOOLBAR_HEIGHT: u32 = 36;
pub const WINDOW_BODY_MARGIN: u32 = 8;
