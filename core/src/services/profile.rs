use crate::config::models::{HardwareProfile, Keymap, VirtualLayout};
use crate::config::{ConfigManager, StorageError};
use thiserror::Error;

use super::traits::ProfileServiceTrait;

#[derive(Error, Debug)]
pub enum ProfileServiceError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub struct ProfileService {
    config_manager: ConfigManager,
}

impl Default for ProfileService {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileService {
    pub fn new() -> Self {
        Self {
            config_manager: ConfigManager::default(),
        }
    }
}

impl ProfileServiceTrait for ProfileService {
    fn list_virtual_layouts(&self) -> Result<Vec<VirtualLayout>, ProfileServiceError> {
        self.config_manager
            .load_virtual_layouts()
            .map(|m| m.into_values().collect())
            .map_err(Into::into)
    }

    fn save_virtual_layout(
        &self,
        layout: VirtualLayout,
    ) -> Result<VirtualLayout, ProfileServiceError> {
        self.config_manager.save_virtual_layout(&layout)?;
        Ok(layout)
    }

    fn delete_virtual_layout(&self, id: &str) -> Result<(), ProfileServiceError> {
        self.config_manager.delete_virtual_layout(id)?;
        Ok(())
    }

    fn list_hardware_profiles(&self) -> Result<Vec<HardwareProfile>, ProfileServiceError> {
        self.config_manager
            .load_hardware_profiles()
            .map(|m| m.into_values().collect())
            .map_err(Into::into)
    }

    fn save_hardware_profile(
        &self,
        profile: HardwareProfile,
    ) -> Result<HardwareProfile, ProfileServiceError> {
        self.config_manager.save_hardware_profile(&profile)?;
        Ok(profile)
    }

    fn delete_hardware_profile(&self, id: &str) -> Result<(), ProfileServiceError> {
        self.config_manager.delete_hardware_profile(id)?;
        Ok(())
    }

    fn list_keymaps(&self) -> Result<Vec<Keymap>, ProfileServiceError> {
        self.config_manager
            .load_keymaps()
            .map(|m| m.into_values().collect())
            .map_err(Into::into)
    }

    fn save_keymap(&self, keymap: Keymap) -> Result<Keymap, ProfileServiceError> {
        self.config_manager.save_keymap(&keymap)?;
        Ok(keymap)
    }

    fn delete_keymap(&self, id: &str) -> Result<(), ProfileServiceError> {
        self.config_manager.delete_keymap(id)?;
        Ok(())
    }
}
