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
use crate::data::preferences::PreferencesDataSource;
use crate::model::{Preferences, Preset};
use crate::prelude::*;
use std::sync::Mutex;

pub struct FakePreferencesDataSource {
    preferences: Mutex<Preferences>,
}

impl FakePreferencesDataSource {
    pub fn new() -> Self {
        let preferences = Preferences {
            presets: vec![Preset {
                name: "Preset 1".to_string(),
                font_size: 24,
                enable_paths: vec!["./arial.ttf".to_string()],
            }],
        };

        Self {
            preferences: Mutex::new(preferences),
        }
    }
}

impl PreferencesDataSource for FakePreferencesDataSource {
    fn retrieve(&self) -> Fallible<Preferences> {
        let prefs = self.preferences.lock().unwrap();
        Ok(Preferences {
            presets: prefs
                .presets
                .iter()
                .map(|p| Preset {
                    name: p.name.clone(),
                    font_size: p.font_size,
                    enable_paths: p.enable_paths.clone(),
                })
                .collect(),
        })
    }

    fn save(&self, preferences: Preferences) -> Fallible<()> {
        *self.preferences.lock().unwrap() = preferences;
        Ok(())
    }
}
