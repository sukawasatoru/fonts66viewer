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
use crate::model::Preferences;
use crate::prelude::*;
use directories::ProjectDirs;

#[cfg(test)]
mod fake_preferences;

mod local_preferences;

trait PreferencesDataSource {
    fn retrieve(&self) -> Fallible<Preferences>;

    fn save(&self, preferences: Preferences) -> Fallible<()>;
}

pub struct PreferencesRepository {
    #[cfg(not(test))]
    data_source: local_preferences::LocalPreferencesDataSource,

    #[cfg(test)]
    data_source: fake_preferences::FakePreferencesDataSource,
}

impl PreferencesRepository {
    #[cfg_attr(test, allow(unused_variables))]
    pub fn new(project: &ProjectDirs) -> Self {
        #[cfg(not(test))]
        return Self {
            data_source: local_preferences::LocalPreferencesDataSource::new(project),
        };

        #[cfg(test)]
        return Self {
            data_source: fake_preferences::FakePreferencesDataSource::new(),
        };
    }

    pub fn retrieve(&self) -> Fallible<Preferences> {
        self.data_source.retrieve()
    }

    pub fn save(&self, preferences: Preferences) -> Fallible<()> {
        self.data_source.save(preferences)
    }
}
