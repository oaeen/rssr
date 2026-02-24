mod core;

use core::AppServices;
use std::collections::BTreeMap;

#[derive(Default)]
struct SharedState {
    services: AppServices,
}

#[tauri::command]
fn app_health(state: tauri::State<'_, SharedState>) -> BTreeMap<String, String> {
    state.services.health_report()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(SharedState::default())
        .invoke_handler(tauri::generate_handler![app_health])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::core::AppServices;

    #[test]
    fn app_services_smoke_test() {
        let report = AppServices::default().health_report();
        assert_eq!(report.get("feed"), Some(&String::from("ready")));
        assert_eq!(report.get("importer"), Some(&String::from("ready")));
        assert_eq!(report.get("subscription"), Some(&String::from("ready")));
        assert_eq!(report.get("llm"), Some(&String::from("ready")));
        assert_eq!(report.get("storage"), Some(&String::from("ready")));
        assert_eq!(report.get("sync"), Some(&String::from("ready")));
    }
}
