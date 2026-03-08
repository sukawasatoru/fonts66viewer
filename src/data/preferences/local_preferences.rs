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
use crate::model::{Preferences, Preset, SQLiteUserVersion};
use crate::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

pub struct LocalPreferencesDataSource {
    mutex: Mutex<()>,
    pathname: PathBuf,
}

impl LocalPreferencesDataSource {
    #[cfg_attr(test, allow(unused))]
    pub fn new(project: &ProjectDirs) -> Self {
        Self::with_pathname(project.config_dir().join("preferences.toml"))
    }

    fn with_pathname(pathname: PathBuf) -> Self {
        Self {
            mutex: Mutex::new(()),
            pathname,
        }
    }

    fn lock(&self) -> MutexGuard<'_, ()> {
        match self.mutex.lock() {
            Ok(data) => data,
            Err(e) => {
                warn!(?e, "try clear poisoned state");
                self.mutex.clear_poison();
                self.mutex.lock().expect("poisoned")
            }
        }
    }

    fn write_dto(&self, dto: &PreferencesDTO) -> Fallible<()> {
        let parent_dir = self
            .pathname
            .parent()
            .context("pathname should have parent")?;
        std::fs::create_dir_all(parent_dir)?;

        let mut writer = BufWriter::new(std::fs::File::create(&self.pathname)?);
        writer.write_all(toml::to_string(dto)?.as_bytes())?;
        writer.flush()?;

        Ok(())
    }

    /// Migrate preferences from an older version to the current version.
    ///
    /// To add a new migration step, add an `if *_file_version < "x.y.z".parse()? { ... }` block
    /// below. When adding new fields to DTOs, use `#[serde(default)]` to maintain compatibility
    /// with older files.
    ///
    /// If a future migration requires destructive changes that cannot be handled by
    /// `#[serde(default)]` (e.g., field type changes or structural reorganization), use
    /// `toml::Value` for that specific migration step only.
    fn migrate(
        &self,
        mut dto: PreferencesDTO,
        _file_version: &SQLiteUserVersion,
        current_version: &SQLiteUserVersion,
    ) -> Fallible<PreferencesDTO> {
        info!(%_file_version, %current_version, "migrating preferences");

        // if *_file_version < "0.2.0".parse()? {
        //     for preset in &mut dto.presets {
        //         // apply migration for 0.2.0
        //     }
        // }

        dto.version = current_version.to_string();
        Ok(dto)
    }
}

impl PreferencesDataSource for LocalPreferencesDataSource {
    /// Retrieve preferences from the TOML file.
    ///
    /// Migration is lazily executed during retrieval (no explicit migration command at app startup).
    /// Version comparison uses `SQLiteUserVersion` (`FromStr` + `PartialOrd`), which packs semver
    /// into a `u32`.
    fn retrieve(&self) -> Fallible<Preferences> {
        let _guard = self.lock();

        if !self.pathname.exists() {
            return Ok(Preferences { presets: vec![] });
        }

        let mut content = String::new();
        BufReader::new(std::fs::File::open(&self.pathname)?).read_to_string(&mut content)?;

        let dto: PreferencesDTO = toml::from_str(&content)?;

        let current_version: SQLiteUserVersion = env!("CARGO_PKG_VERSION").parse()?;
        let file_version: SQLiteUserVersion = dto.version.parse()?;

        if file_version < current_version {
            let migrated_dto = self.migrate(dto, &file_version, &current_version)?;
            self.write_dto(&migrated_dto)?;
            Ok(migrated_dto.into())
        } else {
            Ok(dto.into())
        }
    }

    fn save(&self, preferences: Preferences) -> Fallible<()> {
        let _guard = self.lock();

        let dto = PreferencesDTO::from(preferences);
        self.write_dto(&dto)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct PreferencesDTO {
    version: String,
    presets: Vec<PresetDTO>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PresetDTO {
    pub name: String,
    pub font_size: u32,
    pub enable_paths: Vec<String>,
}

impl From<PreferencesDTO> for Preferences {
    fn from(dto: PreferencesDTO) -> Self {
        Preferences {
            presets: dto.presets.into_iter().map(|p| p.into()).collect(),
        }
    }
}

impl From<Preferences> for PreferencesDTO {
    fn from(prefs: Preferences) -> Self {
        PreferencesDTO {
            version: env!("CARGO_PKG_VERSION").to_string(),
            presets: prefs.presets.into_iter().map(|p| p.into()).collect(),
        }
    }
}

impl From<PresetDTO> for Preset {
    fn from(dto: PresetDTO) -> Self {
        Preset {
            name: dto.name,
            font_size: dto.font_size,
            enable_paths: dto.enable_paths,
        }
    }
}

impl From<Preset> for PresetDTO {
    fn from(preset: Preset) -> Self {
        PresetDTO {
            name: preset.name,
            font_size: preset.font_size,
            enable_paths: preset.enable_paths,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieve_returns_empty_when_file_not_exists() {
        let dir = tempfile::tempdir().unwrap();
        let ds = LocalPreferencesDataSource::with_pathname(dir.path().join("prefs.toml"));
        let prefs = ds.retrieve().unwrap();
        assert!(prefs.presets.is_empty());
    }

    #[test]
    fn save_and_retrieve_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let ds = LocalPreferencesDataSource::with_pathname(dir.path().join("prefs.toml"));
        let prefs = Preferences {
            presets: vec![Preset {
                name: "test".to_string(),
                font_size: 16,
                enable_paths: vec!["/path".to_string()],
            }],
        };
        ds.save(prefs).unwrap();
        let loaded = ds.retrieve().unwrap();
        assert_eq!(loaded.presets.len(), 1);
        assert_eq!(loaded.presets[0].name, "test");
        assert_eq!(loaded.presets[0].font_size, 16);
        assert_eq!(loaded.presets[0].enable_paths, vec!["/path".to_string()]);
    }

    #[test]
    fn save_creates_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let ds = LocalPreferencesDataSource::with_pathname(
            dir.path().join("nested").join("dir").join("prefs.toml"),
        );
        let prefs = Preferences { presets: vec![] };
        ds.save(prefs).unwrap();
        let loaded = ds.retrieve().unwrap();
        assert!(loaded.presets.is_empty());
    }

    #[test]
    fn retrieve_migrates_old_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("prefs.toml");

        std::fs::write(
            &path,
            "version = \"0.1.0\"\n\n\
             [[presets]]\n\
             name = \"old\"\n\
             font_size = 12\n\
             enable_paths = [\"/old\"]\n",
        )
        .unwrap();

        let ds = LocalPreferencesDataSource::with_pathname(path.clone());
        let prefs = ds.retrieve().unwrap();

        assert_eq!(prefs.presets.len(), 1);
        assert_eq!(prefs.presets[0].name, "old");
        assert_eq!(prefs.presets[0].font_size, 12);

        let content = std::fs::read_to_string(&path).unwrap();
        let dto: toml::Value = toml::from_str(&content).unwrap();
        assert_eq!(dto["version"].as_str().unwrap(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn retrieve_skips_migration_when_version_matches() {
        let dir = tempfile::tempdir().unwrap();
        let ds = LocalPreferencesDataSource::with_pathname(dir.path().join("prefs.toml"));
        let prefs = Preferences {
            presets: vec![Preset {
                name: "current".to_string(),
                font_size: 20,
                enable_paths: vec![],
            }],
        };
        ds.save(prefs).unwrap();

        let original_content = std::fs::read_to_string(ds.pathname.clone()).unwrap();
        let loaded = ds.retrieve().unwrap();
        let after_content = std::fs::read_to_string(&ds.pathname).unwrap();

        assert_eq!(loaded.presets[0].name, "current");
        assert_eq!(original_content, after_content);
    }
}
