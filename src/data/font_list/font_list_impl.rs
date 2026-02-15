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

use crate::model::FontEntry;
use serde::Deserialize;

#[derive(Default)]
pub struct FontListRepositoryImpl {
    cache: Vec<FontEntry>,
}

impl FontListRepositoryImpl {
    pub fn new() -> Self {
        let config_string = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/font_list.toml",
        ));

        let cache = toml::from_str::<FontListConfig>(config_string)
            .expect("font_list.toml is invalid")
            .entry
            .into_iter()
            .map(|dto| FontEntry {
                filepath: dto.filepath,
                display_name: dto.display_name,
                font_name: Box::leak(dto.name.into_boxed_str()),
            })
            .collect();

        Self { cache }
    }

    pub fn find_all(&self) -> Vec<FontEntry> {
        self.cache.clone()
    }
}

#[derive(Deserialize)]
struct FontListConfig {
    entry: Vec<FontEntryDTO>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct FontEntryDTO {
    filepath: String,
    display_name: Option<String>,
    name: String,
}
