use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/azookey.rs"));
    include!(concat!(env!("OUT_DIR"), "/window.rs"));
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("azookey_service_descriptor");
}

fn get_config_root() -> PathBuf {
    let appdata = PathBuf::from(std::env::var("APPDATA").unwrap());
    appdata.join("Azookey")
}

const SETTINGS_FILENAME: &str = "settings.json";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ZenzaiConfig {
    pub enable: bool,
    pub profile: String,
    pub backend: String,
    #[serde(default = "default_gpu_layers")]
    pub gpu_layers: u32,
}

fn default_gpu_layers() -> u32 { 99 }

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct UserDictionaryEntry {
    pub reading: String,
    pub word: String,
}

impl UserDictionaryEntry {
    pub fn new(reading: &str, word: &str) -> Result<Self, String> {
        let reading: String = reading.trim().chars().map(|c| {
            if ('\u{30a1}'..='\u{30f6}').contains(&c) {
                char::from_u32(c as u32 - 0x60).unwrap()
            } else { c }
        }).collect();
        let word = word.trim().to_string();
        if reading.is_empty() || reading.chars().count() > 64
            || !reading.chars().all(|c| ('\u{3041}'..='\u{3096}').contains(&c) || c == 'ー') {
            return Err("読みは64文字以内のひらがな・カタカナで入力してください".into());
        }
        if word.is_empty() || word.chars().count() > 128 || word.chars().any(char::is_control) {
            return Err("単語は改行を含まない128文字以内で入力してください".into());
        }
        Ok(Self { reading, word })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub version: String,
    pub zenzai: ZenzaiConfig,
    #[serde(default)]
    pub user_dictionary: Vec<UserDictionaryEntry>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            version: "0.1.0".to_string(),
            zenzai: ZenzaiConfig {
                enable: false,
                profile: "".to_string(),
                backend: "cpu".to_string(),
                gpu_layers: default_gpu_layers(),
            },
            user_dictionary: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn write(&self) {
        self.try_write().expect("Failed to save settings");
    }

    pub fn try_write(&self) -> Result<(), String> {
        let root = get_config_root();
        std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let config_str = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let temporary = root.join(format!("settings.{}.tmp", std::process::id()));
        std::fs::write(&temporary, config_str).map_err(|e| e.to_string())?;
        std::fs::rename(&temporary, root.join(SETTINGS_FILENAME)).map_err(|e| e.to_string())
    }

    pub fn read() -> Self {
        let config_path = get_config_root().join(SETTINGS_FILENAME);
        if !config_path.exists() {
            return AppConfig::default();
        }
        let config_str = std::fs::read_to_string(config_path).unwrap();
        serde_json::from_str(&config_str).unwrap()
    }

    pub fn new() -> Self {
        let config_path = get_config_root();
        if !config_path.exists() {
            std::fs::create_dir_all(&config_path).unwrap();
        }
        let config = AppConfig::read();
        config.write();
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_settings_remain_compatible() {
        let config: AppConfig = serde_json::from_str(r#"{"version":"0.1.0","zenzai":{"enable":false,"profile":"","backend":"vulkan"}}"#).unwrap();
        assert!(config.user_dictionary.is_empty());
        assert_eq!(config.zenzai.gpu_layers, 99);
    }

    #[test]
    fn dictionary_normalizes_and_validates_input() {
        assert_eq!(UserDictionaryEntry::new(" アズーキー ", " azooKey ").unwrap(),
            UserDictionaryEntry { reading: "あずーきー".into(), word: "azooKey".into() });
        for reading in ["", "abc", "あ い", "漢字", "ｱｽﾞｰｷｰ"] {
            assert!(UserDictionaryEntry::new(reading, "単語").is_err());
        }
        assert!(UserDictionaryEntry::new("あ", "a\nb").is_err());
        assert!(UserDictionaryEntry::new(&"あ".repeat(65), "単語").is_err());
        assert!(UserDictionaryEntry::new("あ", &"字".repeat(129)).is_err());
    }
}
