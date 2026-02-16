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

#[cfg(test)]
mod fake_font_list;

mod font_list_impl;

trait FontListDataSource {
    fn find_all(&self) -> Vec<FontEntry>;
}

pub struct FontListRepository {
    #[cfg(not(test))]
    data_source: font_list_impl::FontListDataSourceImpl,

    #[cfg(test)]
    data_source: fake_font_list::FakeFontListDataSource,
}

impl FontListRepository {
    pub fn find_all(&self) -> Vec<FontEntry> {
        self.data_source.find_all()
    }
}

impl Default for FontListRepository {
    #[cfg(not(test))]
    fn default() -> Self {
        Self {
            data_source: font_list_impl::FontListDataSourceImpl::new(),
        }
    }

    #[cfg(test)]
    fn default() -> Self {
        Self {
            data_source: fake_font_list::FakeFontListDataSource::new(),
        }
    }
}
