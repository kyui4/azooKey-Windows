mod ipc;

use serde::{Deserialize, Serialize};
use shared::{AppConfig, UserDictionaryEntry};
use std::{path::PathBuf, sync::Mutex};

#[derive(Debug)]
pub struct AppState {
    settings: Mutex<AppConfig>,
    ipc: Option<ipc::IPCService>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            settings: Mutex::new(AppConfig::new()),
            ipc: ipc::IPCService::new().ok(),
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> AppConfig {
    let config = state.settings.lock().unwrap();
    config.clone()
}

#[tauri::command]
fn update_config(state: tauri::State<AppState>, mut new_config: AppConfig) -> Result<(), String> {
    let mut config = state.settings.lock().unwrap();
    // Dictionary commands own this field; stale settings forms must not overwrite it.
    new_config.user_dictionary = config.user_dictionary.clone();
    if !["cpu", "cuda", "vulkan"].contains(&new_config.zenzai.backend.as_str())
        || new_config.zenzai.gpu_layers > 99 {
        return Err("バックエンドまたはGPU層数が不正です".into());
    }
    new_config.try_write()?;
    *config = new_config;
    notify_converter(&state).map_err(|e| format!("設定は保存済みですが、変換エンジンへ反映できませんでした: {e}"))
}

fn notify_converter(state: &AppState) -> Result<(), String> {
    let mut ipc = match &state.ipc {
        Some(ipc) => ipc.clone(),
        None => ipc::IPCService::new().map_err(|e| e.to_string())?,
    };
    ipc.update_config().map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct DictionaryUpdate {
    entries: Vec<UserDictionaryEntry>,
    applied: bool,
}

#[tauri::command]
fn get_dictionary(state: tauri::State<AppState>) -> Vec<UserDictionaryEntry> {
    state.settings.lock().unwrap().user_dictionary.clone()
}

fn save_dictionary(state: &AppState, config: &mut AppConfig, entries: Vec<UserDictionaryEntry>) -> Result<DictionaryUpdate, String> {
    let mut updated = config.clone();
    updated.user_dictionary = entries;
    updated.try_write()?;
    *config = updated;
    Ok(DictionaryUpdate {
        entries: config.user_dictionary.clone(),
        applied: notify_converter(state).is_ok(),
    })
}

#[tauri::command]
fn add_dictionary_entry(state: tauri::State<AppState>, reading: String, word: String) -> Result<DictionaryUpdate, String> {
    let entry = UserDictionaryEntry::new(&reading, &word)?;
    let mut config = state.settings.lock().unwrap();
    let mut entries = config.user_dictionary.clone();
    if entries.contains(&entry) { return Err("同じ読みと単語は登録済みです".into()); }
    // The pinned converter uses a linear lookup for dynamic dictionaries.
    if entries.len() >= 1000 { return Err("辞書登録は1000件までです".into()); }
    entries.push(entry);
    save_dictionary(&state, &mut config, entries)
}

#[tauri::command]
fn delete_dictionary_entry(state: tauri::State<AppState>, reading: String, word: String) -> Result<DictionaryUpdate, String> {
    let mut config = state.settings.lock().unwrap();
    let entries = config.user_dictionary.iter().filter(|entry| entry.reading != reading || entry.word != word).cloned().collect();
    save_dictionary(&state, &mut config, entries)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Capability {
    cpu: bool,
    cuda: bool,
    vulkan: bool,
}

#[tauri::command]
fn check_capability() -> Capability {
    // cuda:
    // cudart64_12.dll
    // cublas64_12.dll

    // vulkan:
    // vulkan-1.dllの存在確認

    let mut capability = Capability {
        cpu: true,
        cuda: false,
        vulkan: false,
    };

    // Check for CUDA availability
    let cuda_files = ["cudart64_12.dll", "cublas64_12.dll"];
    let cuda_available = cuda_files.iter().all(|file| {
        // Check if the file exists in system path or in the current directory
        std::env::var("PATH")
            .unwrap_or_default()
            .split(';')
            .map(PathBuf::from)
            .chain(std::iter::once(std::env::current_dir().unwrap_or_default()))
            .any(|path| path.join(file).exists())
    });
    capability.cuda = cuda_available;

    // Check for Vulkan availability
    let vulkan_file = "vulkan-1.dll";
    let vulkan_available = std::env::var("PATH")
        .unwrap_or_default()
        .split(';')
        .map(PathBuf::from)
        .chain(std::iter::once(std::env::current_dir().unwrap_or_default()))
        .chain(std::env::var_os("WINDIR").map(|dir| PathBuf::from(dir).join("System32")))
        .any(|path| path.join(vulkan_file).exists());
    capability.vulkan = vulkan_available;

    capability
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_config,
            update_config,
            check_capability,
            get_dictionary,
            add_dictionary_entry,
            delete_dictionary_entry
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
