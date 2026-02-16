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
use crate::data::font_list::FontListDataSource;
use crate::model::FontEntry;

pub struct FakeFontListDataSource {
    list: Vec<FontEntry>,
}

impl FakeFontListDataSource {
    pub fn new() -> Self {
        let mut list = vec![];
        list.push(FontEntry {
            filepath: "./arial.ttf".to_string(),
            display_name: None,
            font_name: "Arial",
        });

        Self { list }
    }
}

impl FontListDataSource for FakeFontListDataSource {
    fn find_all(&self) -> Vec<FontEntry> {
        self.list.clone()
    }
}
