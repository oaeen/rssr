mod core;

use core::AppServices;
use core::importer::{
    build_import_preview, normalize_url, parse_json_sources, parse_opml, parse_url_list,
    ImportSource,
};
use core::storage::models::{NewSource, SourceRecord};
use core::storage::repository::SourceRepository;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use tauri::Manager;

struct SharedState {
    services: AppServices,
    source_repository: SourceRepository,
}

#[derive(Debug, Clone, Deserialize)]
struct UpsertSourceRequest {
    title: String,
    site_url: Option<String>,
    feed_url: String,
    category: Option<String>,
    is_active: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct ImportRequest {
    format: String,
    content: String,
    default_category: Option<String>,
    is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
struct SourceDto {
    id: i64,
    title: String,
    site_url: Option<String>,
    feed_url: String,
    category: Option<String>,
    is_active: bool,
    failure_count: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
struct ImportPreviewResponse {
    new_count: usize,
    duplicate_count: usize,
    new_sources: Vec<ImportSource>,
    duplicate_sources: Vec<ImportSource>,
}

#[derive(Debug, Clone, Serialize)]
struct ImportExecuteResponse {
    imported_count: usize,
    duplicate_count: usize,
}

#[tauri::command]
fn app_health(state: tauri::State<'_, SharedState>) -> BTreeMap<String, String> {
    state.services.health_report()
}

#[tauri::command]
async fn list_sources(state: tauri::State<'_, SharedState>) -> Result<Vec<SourceDto>, String> {
    let rows = state
        .source_repository
        .list_sources()
        .await
        .map_err(|error| error.to_string())?;
    Ok(rows.into_iter().map(source_to_dto).collect())
}

#[tauri::command]
async fn upsert_source(
    request: UpsertSourceRequest,
    state: tauri::State<'_, SharedState>,
) -> Result<SourceDto, String> {
    let source = NewSource {
        title: request.title,
        site_url: request.site_url,
        feed_url: request.feed_url,
        category: request.category,
        is_active: request.is_active,
    };
    let row = state
        .source_repository
        .upsert_source(&source)
        .await
        .map_err(|error| error.to_string())?;
    Ok(source_to_dto(row))
}

#[tauri::command]
async fn delete_source(id: i64, state: tauri::State<'_, SharedState>) -> Result<u64, String> {
    state
        .source_repository
        .delete_source(id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn set_sources_active(
    source_ids: Vec<i64>,
    is_active: bool,
    state: tauri::State<'_, SharedState>,
) -> Result<u64, String> {
    state
        .source_repository
        .set_sources_active(&source_ids, is_active)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn preview_import(
    request: ImportRequest,
    state: tauri::State<'_, SharedState>,
) -> Result<ImportPreviewResponse, String> {
    let candidates = parse_import_sources(&request)?;
    let existing_rows = state
        .source_repository
        .list_sources()
        .await
        .map_err(|error| error.to_string())?;
    let existing_feed_urls: HashSet<String> = existing_rows
        .into_iter()
        .map(|row| normalize_url(&row.feed_url))
        .collect();
    let preview = build_import_preview(candidates, &existing_feed_urls);

    Ok(ImportPreviewResponse {
        new_count: preview.new_sources.len(),
        duplicate_count: preview.duplicate_sources.len(),
        new_sources: preview.new_sources,
        duplicate_sources: preview.duplicate_sources,
    })
}

#[tauri::command]
async fn import_sources(
    request: ImportRequest,
    state: tauri::State<'_, SharedState>,
) -> Result<ImportExecuteResponse, String> {
    let candidates = parse_import_sources(&request)?;
    let existing_rows = state
        .source_repository
        .list_sources()
        .await
        .map_err(|error| error.to_string())?;
    let existing_feed_urls: HashSet<String> = existing_rows
        .into_iter()
        .map(|row| normalize_url(&row.feed_url))
        .collect();
    let preview = build_import_preview(candidates, &existing_feed_urls);
    let is_active = request.is_active.unwrap_or(true);
    let default_category = request.default_category;
    let sources_to_import: Vec<NewSource> = preview
        .new_sources
        .iter()
        .map(|source| NewSource {
            title: source.title.clone(),
            site_url: source.site_url.clone(),
            feed_url: source.feed_url.clone(),
            category: source.category.clone().or_else(|| default_category.clone()),
            is_active,
        })
        .collect();

    let imported_count = state
        .source_repository
        .upsert_sources_batch(&sources_to_import)
        .await
        .map_err(|error| error.to_string())?;

    Ok(ImportExecuteResponse {
        imported_count,
        duplicate_count: preview.duplicate_sources.len(),
    })
}

fn parse_import_sources(request: &ImportRequest) -> Result<Vec<ImportSource>, String> {
    match request.format.to_lowercase().as_str() {
        "opml" | "xml" => parse_opml(&request.content).map_err(|error| error.to_string()),
        "url_list" | "urls" | "txt" => Ok(parse_url_list(&request.content)),
        "json" | "json_list" => parse_json_sources(&request.content).map_err(|error| error.to_string()),
        unsupported => Err(format!("unsupported import format: {unsupported}")),
    }
}

fn source_to_dto(source: SourceRecord) -> SourceDto {
    SourceDto {
        id: source.id,
        title: source.title,
        site_url: source.site_url,
        feed_url: source.feed_url,
        category: source.category,
        is_active: source.is_active == 1,
        failure_count: source.failure_count,
        created_at: source.created_at,
        updated_at: source.updated_at,
    }
}

fn build_database_url(app_handle: &tauri::AppHandle) -> Result<String, std::io::Error> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    std::fs::create_dir_all(&app_data_dir)?;
    let database_path = app_data_dir.join("rssr.db");
    Ok(to_sqlite_url(database_path))
}

fn to_sqlite_url(path: PathBuf) -> String {
    format!("sqlite://{}?mode=rwc", path.to_string_lossy())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let database_url = build_database_url(app.handle())?;
            let repository = tauri::async_runtime::block_on(SourceRepository::connect(&database_url))
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            app.manage(SharedState {
                services: AppServices::default(),
                source_repository: repository,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_health,
            list_sources,
            upsert_source,
            delete_source,
            set_sources_active,
            preview_import,
            import_sources
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::parse_import_sources;
    use super::ImportRequest;

    #[test]
    fn import_format_parser_accepts_known_aliases() {
        let payload = ImportRequest {
            format: "urls".to_string(),
            content: "https://example.com/feed.xml".to_string(),
            default_category: None,
            is_active: Some(true),
        };
        let parsed = parse_import_sources(&payload).expect("url alias should parse");
        assert_eq!(parsed.len(), 1);
    }
}
