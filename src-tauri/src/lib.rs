mod core;

use core::AppServices;
use core::feed::fetcher::{fetch_feed_with_retry, FetchStatus};
use core::feed::parser::parse_feed_bytes;
use core::importer::{
    build_import_preview, normalize_url, parse_json_sources, parse_opml, parse_url_list,
    ImportSource,
};
use core::llm::{call_chat_completion, validate_config, LlmConfig};
use core::storage::models::{EntryRecord, NewSource, SourceRecord};
use core::storage::repository::SourceRepository;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;
use tauri::Manager;

const LLM_CONFIG_KEY: &str = "llm_config";

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

#[derive(Debug, Clone, Deserialize)]
struct ListEntriesRequest {
    source_id: Option<i64>,
    search: Option<String>,
    unread_only: bool,
    limit: Option<i64>,
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
    etag: Option<String>,
    last_modified: Option<String>,
    last_synced_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
struct EntryDto {
    id: i64,
    source_id: i64,
    source_title: String,
    guid: Option<String>,
    link: String,
    title: String,
    summary: Option<String>,
    content: Option<String>,
    published_at: Option<String>,
    is_read: bool,
    is_starred: bool,
    created_at: String,
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

#[derive(Debug, Clone, Serialize)]
struct SyncSourceResponse {
    source_id: i64,
    status: String,
    upserted_entries: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SyncBatchResponse {
    synced_sources: usize,
    failed_sources: usize,
    total_upserted_entries: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct TranslateRequest {
    entry_id: i64,
    target_language: String,
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

#[tauri::command]
async fn list_entries(
    request: ListEntriesRequest,
    state: tauri::State<'_, SharedState>,
) -> Result<Vec<EntryDto>, String> {
    let rows = state
        .source_repository
        .list_entries(
            request.source_id,
            request.search.as_deref(),
            request.unread_only,
            request.limit.unwrap_or(300),
        )
        .await
        .map_err(|error| error.to_string())?;
    Ok(rows.into_iter().map(entry_to_dto).collect())
}

#[tauri::command]
async fn mark_entry_read(
    entry_id: i64,
    is_read: bool,
    state: tauri::State<'_, SharedState>,
) -> Result<u64, String> {
    state
        .source_repository
        .mark_entry_read(entry_id, is_read)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn sync_source(source_id: i64, state: tauri::State<'_, SharedState>) -> Result<SyncSourceResponse, String> {
    let source = state
        .source_repository
        .get_source_by_id(source_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("source {source_id} not found"))?;
    sync_single_source(&state.source_repository, source).await
}

#[tauri::command]
async fn sync_active_sources(state: tauri::State<'_, SharedState>) -> Result<SyncBatchResponse, String> {
    sync_active_sources_internal(&state.source_repository).await
}

#[tauri::command]
async fn get_llm_config(state: tauri::State<'_, SharedState>) -> Result<Option<LlmConfig>, String> {
    get_saved_or_env_llm_config(&state.source_repository).await
}

#[tauri::command]
async fn save_llm_config(
    config: LlmConfig,
    state: tauri::State<'_, SharedState>,
) -> Result<(), String> {
    validate_config(&config).map_err(|error| error.to_string())?;
    let serialized = serde_json::to_string(&config).map_err(|error| error.to_string())?;
    state
        .source_repository
        .set_setting(LLM_CONFIG_KEY, &serialized)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn test_llm_connection(
    config: Option<LlmConfig>,
    state: tauri::State<'_, SharedState>,
) -> Result<String, String> {
    let resolved = resolve_llm_config(config, &state.source_repository).await?;
    let response = call_chat_completion(
        &resolved,
        "You are a connectivity checker.",
        "Reply with exactly: ok",
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok(response)
}

#[tauri::command]
async fn summarize_entry(entry_id: i64, state: tauri::State<'_, SharedState>) -> Result<String, String> {
    let config = resolve_llm_config(None, &state.source_repository).await?;
    let entry = state
        .source_repository
        .get_entry_by_id(entry_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("entry {entry_id} not found"))?;
    let input = build_llm_entry_input(&entry);
    let hash = hash_llm_input("summary", &config.model, &input);
    if let Some(cached) = state
        .source_repository
        .get_llm_cache("summary", &config.model, &hash)
        .await
        .map_err(|error| error.to_string())?
    {
        return Ok(cached);
    }

    let output = call_chat_completion(
        &config,
        "You summarize technical articles in concise Chinese.",
        &format!("请总结下面这篇文章，输出 5 条以内要点：\n\n{input}"),
    )
    .await
    .map_err(|error| error.to_string())?;
    state
        .source_repository
        .set_llm_cache("summary", &config.model, &hash, &output)
        .await
        .map_err(|error| error.to_string())?;
    Ok(output)
}

#[tauri::command]
async fn translate_entry(
    request: TranslateRequest,
    state: tauri::State<'_, SharedState>,
) -> Result<String, String> {
    let config = resolve_llm_config(None, &state.source_repository).await?;
    let entry = state
        .source_repository
        .get_entry_by_id(request.entry_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("entry {} not found", request.entry_id))?;
    let input = build_llm_entry_input(&entry);
    let task_type = format!("translate:{}", request.target_language.to_lowercase());
    let hash = hash_llm_input(&task_type, &config.model, &input);
    if let Some(cached) = state
        .source_repository
        .get_llm_cache(&task_type, &config.model, &hash)
        .await
        .map_err(|error| error.to_string())?
    {
        return Ok(cached);
    }

    let output = call_chat_completion(
        &config,
        "You are a professional technical translator.",
        &format!(
            "Translate the following article into {}. Keep formatting simple and readable.\n\n{}",
            request.target_language, input
        ),
    )
    .await
    .map_err(|error| error.to_string())?;
    state
        .source_repository
        .set_llm_cache(&task_type, &config.model, &hash, &output)
        .await
        .map_err(|error| error.to_string())?;
    Ok(output)
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
        etag: source.etag,
        last_modified: source.last_modified,
        last_synced_at: source.last_synced_at,
        created_at: source.created_at,
        updated_at: source.updated_at,
    }
}

fn entry_to_dto(entry: EntryRecord) -> EntryDto {
    EntryDto {
        id: entry.id,
        source_id: entry.source_id,
        source_title: entry.source_title,
        guid: entry.guid,
        link: entry.link,
        title: entry.title,
        summary: entry.summary,
        content: entry.content,
        published_at: entry.published_at,
        is_read: entry.is_read == 1,
        is_starred: entry.is_starred == 1,
        created_at: entry.created_at,
    }
}

async fn sync_single_source(
    repository: &SourceRepository,
    source: SourceRecord,
) -> Result<SyncSourceResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| error.to_string())?;

    let fetched = fetch_feed_with_retry(
        &client,
        &source.feed_url,
        source.etag.as_deref(),
        source.last_modified.as_deref(),
        2,
    )
    .await;

    let result = match fetched {
        Ok(FetchStatus::NotModified) => {
            repository
                .update_source_sync_success(
                    source.id,
                    source.etag.as_deref(),
                    source.last_modified.as_deref(),
                )
                .await
                .map_err(|error| error.to_string())?;
            SyncSourceResponse {
                source_id: source.id,
                status: "not_modified".to_string(),
                upserted_entries: 0,
            }
        }
        Ok(FetchStatus::Updated(payload)) => {
            let parsed = parse_feed_bytes(&payload.body).map_err(|error| error.to_string())?;
            let upserted_entries = repository
                .upsert_entries(source.id, &parsed.entries)
                .await
                .map_err(|error| error.to_string())?;
            repository
                .update_source_sync_success(
                    source.id,
                    payload.etag.as_deref(),
                    payload.last_modified.as_deref(),
                )
                .await
                .map_err(|error| error.to_string())?;
            SyncSourceResponse {
                source_id: source.id,
                status: "updated".to_string(),
                upserted_entries,
            }
        }
        Err(error) => {
            repository
                .increment_source_failure(source.id)
                .await
                .map_err(|inner| inner.to_string())?;
            return Err(error.to_string());
        }
    };

    Ok(result)
}

async fn sync_active_sources_internal(repository: &SourceRepository) -> Result<SyncBatchResponse, String> {
    let sources = repository
        .list_sync_candidates()
        .await
        .map_err(|error| error.to_string())?;

    let mut synced_sources = 0_usize;
    let mut failed_sources = 0_usize;
    let mut total_upserted_entries = 0_usize;
    for source in sources {
        match sync_single_source(repository, source).await {
            Ok(result) => {
                synced_sources += 1;
                total_upserted_entries += result.upserted_entries;
            }
            Err(_) => {
                failed_sources += 1;
            }
        }
    }

    Ok(SyncBatchResponse {
        synced_sources,
        failed_sources,
        total_upserted_entries,
    })
}

async fn resolve_llm_config(
    provided: Option<LlmConfig>,
    repository: &SourceRepository,
) -> Result<LlmConfig, String> {
    if let Some(config) = provided {
        validate_config(&config).map_err(|error| error.to_string())?;
        return Ok(config);
    }
    let config = get_saved_or_env_llm_config(repository)
        .await?
        .ok_or_else(|| "llm config is missing".to_string())?;
    validate_config(&config).map_err(|error| error.to_string())?;
    Ok(config)
}

async fn get_saved_or_env_llm_config(
    repository: &SourceRepository,
) -> Result<Option<LlmConfig>, String> {
    if let Some(raw) = repository
        .get_setting(LLM_CONFIG_KEY)
        .await
        .map_err(|error| error.to_string())?
    {
        let parsed = serde_json::from_str::<LlmConfig>(&raw).map_err(|error| error.to_string())?;
        return Ok(Some(parsed));
    }

    let base_url = std::env::var("RSSR_LLM_BASE_URL").unwrap_or_default();
    let api_key = std::env::var("RSSR_LLM_API_KEY").unwrap_or_default();
    let model = std::env::var("RSSR_LLM_MODEL").unwrap_or_default();
    if base_url.trim().is_empty() || api_key.trim().is_empty() || model.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(LlmConfig {
        base_url,
        api_key,
        model,
        timeout_secs: 30,
    }))
}

fn build_llm_entry_input(entry: &EntryRecord) -> String {
    let mut blocks = vec![
        format!("Title: {}", entry.title),
        format!("Link: {}", entry.link),
    ];
    if let Some(summary) = &entry.summary {
        blocks.push(format!("Summary: {summary}"));
    }
    if let Some(content) = &entry.content {
        let text = content.chars().take(8000).collect::<String>();
        blocks.push(format!("Content:\n{text}"));
    }
    blocks.join("\n\n")
}

fn hash_llm_input(task_type: &str, model: &str, input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(task_type.as_bytes());
    hasher.update(b"::");
    hasher.update(model.as_bytes());
    hasher.update(b"::");
    hasher.update(input.as_bytes());
    let bytes = hasher.finalize();
    format!("{bytes:x}")
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
            let _ = dotenvy::from_filename(".env.local");
            let database_url = build_database_url(app.handle())?;
            let repository = tauri::async_runtime::block_on(SourceRepository::connect(&database_url))
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            let background_repository = repository.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let _ = sync_active_sources_internal(&background_repository).await;
                    tokio::time::sleep(Duration::from_secs(600)).await;
                }
            });
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
            import_sources,
            list_entries,
            mark_entry_read,
            sync_source,
            sync_active_sources,
            get_llm_config,
            save_llm_config,
            test_llm_connection,
            summarize_entry,
            translate_entry
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::hash_llm_input;
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

    #[test]
    fn llm_input_hash_is_deterministic() {
        let a = hash_llm_input("summary", "deepseek-chat", "hello");
        let b = hash_llm_input("summary", "deepseek-chat", "hello");
        assert_eq!(a, b);
    }
}
