mod core;

use core::feed::fetcher::{fetch_feed_with_retry, FetchStatus};
use core::feed::parser::parse_feed_bytes;
use core::importer::{
    build_import_preview, normalize_url, parse_json_sources, parse_opml, parse_url_list,
    ImportSource,
};
use core::llm::{call_chat_completion, validate_config, LlmConfig};
use core::storage::models::{EntryRecord, NewSource, SourceRecord};
use core::storage::repository::SourceRepository;
use core::AppServices;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;
use tokio::sync::RwLock;
use tokio::task::JoinSet;

const LLM_CONFIG_KEY: &str = "llm_config";
const SYNC_SETTINGS_KEY: &str = "sync_settings";

const DEFAULT_SYNC_INTERVAL_SECS: u64 = 600;
const DEFAULT_SYNC_MAX_CONCURRENCY: u32 = 6;
const DEFAULT_SYNC_BATCH_LIMIT: u32 = 24;
const DEFAULT_SYNC_TIMEOUT_SECS: u64 = 12;
const DEFAULT_SYNC_RETRY_COUNT: u32 = 1;

struct SharedState {
    services: AppServices,
    source_repository: SourceRepository,
    sync_runtime: Arc<SyncRuntime>,
}

struct SyncRuntime {
    running: AtomicBool,
    last_report: RwLock<Option<SyncBatchResponse>>,
    last_error: RwLock<Option<String>>,
}

impl Default for SyncRuntime {
    fn default() -> Self {
        Self {
            running: AtomicBool::new(false),
            last_report: RwLock::new(None),
            last_error: RwLock::new(None),
        }
    }
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
    translated_title: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncSettings {
    interval_secs: u64,
    max_concurrency: u32,
    batch_limit: u32,
    timeout_secs: u64,
    retry_count: u32,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            interval_secs: DEFAULT_SYNC_INTERVAL_SECS,
            max_concurrency: DEFAULT_SYNC_MAX_CONCURRENCY,
            batch_limit: DEFAULT_SYNC_BATCH_LIMIT,
            timeout_secs: DEFAULT_SYNC_TIMEOUT_SECS,
            retry_count: DEFAULT_SYNC_RETRY_COUNT,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct SyncRuntimeStatus {
    running: bool,
    last_report: Option<SyncBatchResponse>,
    last_error: Option<String>,
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
async fn sync_source(
    source_id: i64,
    state: tauri::State<'_, SharedState>,
) -> Result<SyncSourceResponse, String> {
    let source = state
        .source_repository
        .get_source_by_id(source_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("source {source_id} not found"))?;
    let settings = load_sync_settings(&state.source_repository).await?;
    sync_single_source(&state.source_repository, source, &settings).await
}

#[tauri::command]
async fn sync_active_sources(
    state: tauri::State<'_, SharedState>,
) -> Result<SyncRuntimeStatus, String> {
    if state.sync_runtime.running.swap(true, Ordering::SeqCst) {
        return get_sync_runtime_status(state).await;
    }

    let repository = state.source_repository.clone();
    let runtime = state.sync_runtime.clone();
    tauri::async_runtime::spawn(async move {
        let result = sync_active_sources_internal(&repository).await;
        match result {
            Ok(report) => {
                {
                    let mut guard = runtime.last_report.write().await;
                    *guard = Some(report);
                }
                {
                    let mut guard = runtime.last_error.write().await;
                    *guard = None;
                }
                let title_repository = repository.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = translate_titles_background(&title_repository, 60).await;
                });
            }
            Err(error) => {
                let mut guard = runtime.last_error.write().await;
                *guard = Some(error);
            }
        }
        runtime.running.store(false, Ordering::SeqCst);
    });

    get_sync_runtime_status(state).await
}

#[tauri::command]
async fn get_sync_runtime_status(
    state: tauri::State<'_, SharedState>,
) -> Result<SyncRuntimeStatus, String> {
    let last_report = state.sync_runtime.last_report.read().await.clone();
    let last_error = state.sync_runtime.last_error.read().await.clone();
    Ok(SyncRuntimeStatus {
        running: state.sync_runtime.running.load(Ordering::SeqCst),
        last_report,
        last_error,
    })
}

#[tauri::command]
async fn get_sync_settings(state: tauri::State<'_, SharedState>) -> Result<SyncSettings, String> {
    load_sync_settings(&state.source_repository).await
}

#[tauri::command]
async fn save_sync_settings(
    settings: SyncSettings,
    state: tauri::State<'_, SharedState>,
) -> Result<SyncSettings, String> {
    let normalized = normalize_sync_settings(settings);
    let serialized = serde_json::to_string(&normalized).map_err(|error| error.to_string())?;
    state
        .source_repository
        .set_setting(SYNC_SETTINGS_KEY, &serialized)
        .await
        .map_err(|error| error.to_string())?;
    Ok(normalized)
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
async fn summarize_entry(
    entry_id: i64,
    state: tauri::State<'_, SharedState>,
) -> Result<String, String> {
    let config = resolve_llm_config(None, &state.source_repository).await?;
    let entry = state
        .source_repository
        .get_entry_by_id(entry_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("entry {entry_id} not found"))?;
    let article_text = fetch_webpage_text_for_summary(&entry.link, config.timeout_secs)
        .await
        .unwrap_or_else(|_| fallback_entry_text(&entry));
    let input = build_summary_input(&entry, &article_text);
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

fn parse_import_sources(request: &ImportRequest) -> Result<Vec<ImportSource>, String> {
    match request.format.to_lowercase().as_str() {
        "opml" | "xml" => parse_opml(&request.content).map_err(|error| error.to_string()),
        "url_list" | "urls" | "txt" => Ok(parse_url_list(&request.content)),
        "json" | "json_list" => {
            parse_json_sources(&request.content).map_err(|error| error.to_string())
        }
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
        translated_title: entry.translated_title,
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
    settings: &SyncSettings,
) -> Result<SyncSourceResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(settings.timeout_secs))
        .build()
        .map_err(|error| error.to_string())?;

    let fetched = fetch_feed_with_retry(
        &client,
        &source.feed_url,
        source.etag.as_deref(),
        source.last_modified.as_deref(),
        settings.retry_count as usize,
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

async fn sync_active_sources_internal(
    repository: &SourceRepository,
) -> Result<SyncBatchResponse, String> {
    let settings = load_sync_settings(repository).await?;
    let sources = repository
        .list_sync_candidates(settings.batch_limit as i64)
        .await
        .map_err(|error| error.to_string())?;
    let semaphore = Arc::new(tokio::sync::Semaphore::new(
        settings.max_concurrency as usize,
    ));
    let mut join_set = JoinSet::new();
    for source in sources {
        let repo = repository.clone();
        let sem = semaphore.clone();
        let copied_settings = settings.clone();
        join_set.spawn(async move {
            let _permit = sem
                .acquire_owned()
                .await
                .map_err(|error| error.to_string())?;
            sync_single_source(&repo, source, &copied_settings).await
        });
    }
    let mut synced_sources = 0_usize;
    let mut failed_sources = 0_usize;
    let mut total_upserted_entries = 0_usize;

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(report)) => {
                synced_sources += 1;
                total_upserted_entries += report.upserted_entries;
            }
            Ok(Err(_)) | Err(_) => failed_sources += 1,
        }
    }

    Ok(SyncBatchResponse {
        synced_sources,
        failed_sources,
        total_upserted_entries,
    })
}

async fn translate_titles_background(
    repository: &SourceRepository,
    limit: i64,
) -> Result<usize, String> {
    let config = match get_saved_or_env_llm_config(repository).await? {
        Some(config) => config,
        None => return Ok(0),
    };
    validate_config(&config).map_err(|error| error.to_string())?;
    let targets = repository
        .list_entries_without_translated_title(limit)
        .await
        .map_err(|error| error.to_string())?;
    if targets.is_empty() {
        return Ok(0);
    }

    let mut updated = 0_usize;
    for target in targets {
        let input = target.title.trim();
        if input.is_empty() {
            continue;
        }
        let hash = hash_llm_input("title_translate_zh", &config.model, input);
        let translated = if let Some(cached) = repository
            .get_llm_cache("title_translate_zh", &config.model, &hash)
            .await
            .map_err(|error| error.to_string())?
        {
            cached
        } else {
            let result = call_chat_completion(
                &config,
                "You translate English article titles into concise Chinese.",
                &format!(
                    "Translate this article title into Chinese and keep it concise. Output only Chinese title.\n\n{}",
                    input
                ),
            )
            .await
            .map_err(|error| error.to_string())?;
            repository
                .set_llm_cache("title_translate_zh", &config.model, &hash, &result)
                .await
                .map_err(|error| error.to_string())?;
            result
        };

        repository
            .set_entry_translated_title(target.id, translated.trim())
            .await
            .map_err(|error| error.to_string())?;
        updated += 1;
    }

    Ok(updated)
}

async fn load_sync_settings(repository: &SourceRepository) -> Result<SyncSettings, String> {
    if let Some(raw) = repository
        .get_setting(SYNC_SETTINGS_KEY)
        .await
        .map_err(|error| error.to_string())?
    {
        let parsed =
            serde_json::from_str::<SyncSettings>(&raw).map_err(|error| error.to_string())?;
        return Ok(normalize_sync_settings(parsed));
    }
    Ok(SyncSettings::default())
}

fn normalize_sync_settings(settings: SyncSettings) -> SyncSettings {
    SyncSettings {
        interval_secs: settings.interval_secs.clamp(60, 3600),
        max_concurrency: settings.max_concurrency.clamp(1, 16),
        batch_limit: settings.batch_limit.clamp(1, 200),
        timeout_secs: settings.timeout_secs.clamp(5, 60),
        retry_count: settings.retry_count.clamp(0, 4),
    }
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

fn fallback_entry_text(entry: &EntryRecord) -> String {
    let mut blocks = Vec::new();
    if let Some(summary) = &entry.summary {
        blocks.push(summary.clone());
    }
    if let Some(content) = &entry.content {
        blocks.push(content.clone());
    }
    if blocks.is_empty() {
        return entry.title.clone();
    }
    blocks.join("\n\n")
}

fn build_summary_input(entry: &EntryRecord, article_text: &str) -> String {
    let body = article_text.chars().take(12000).collect::<String>();
    format!(
        "Title: {}\nLink: {}\n\nArticle Text:\n{}",
        entry.title, entry.link, body
    )
}

async fn fetch_webpage_text_for_summary(link: &str, timeout_secs: u64) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs.max(6)))
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .get(link)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "fetch webpage status: {}",
            response.status().as_u16()
        ));
    }
    let html = response.text().await.map_err(|error| error.to_string())?;
    let text = html2text::from_read(html.as_bytes(), 120);
    let normalized = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(1200)
        .collect::<Vec<_>>()
        .join("\n");
    if normalized.is_empty() {
        return Err("empty article text".to_string());
    }
    Ok(normalized)
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
            let repository =
                tauri::async_runtime::block_on(SourceRepository::connect(&database_url))
                    .map_err(|error| std::io::Error::other(error.to_string()))?;
            let background_repository = repository.clone();
            let sync_runtime = Arc::new(SyncRuntime::default());
            let background_runtime = sync_runtime.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    if !background_runtime.running.swap(true, Ordering::SeqCst) {
                        let result = sync_active_sources_internal(&background_repository).await;
                        match result {
                            Ok(report) => {
                                {
                                    let mut guard = background_runtime.last_report.write().await;
                                    *guard = Some(report);
                                }
                                {
                                    let mut guard = background_runtime.last_error.write().await;
                                    *guard = None;
                                }
                                let _ =
                                    translate_titles_background(&background_repository, 60).await;
                            }
                            Err(error) => {
                                let mut guard = background_runtime.last_error.write().await;
                                *guard = Some(error);
                            }
                        }
                        background_runtime.running.store(false, Ordering::SeqCst);
                    }

                    let settings = load_sync_settings(&background_repository)
                        .await
                        .unwrap_or_default();
                    tokio::time::sleep(Duration::from_secs(settings.interval_secs)).await;
                }
            });
            app.manage(SharedState {
                services: AppServices::default(),
                source_repository: repository,
                sync_runtime,
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
            get_sync_runtime_status,
            get_sync_settings,
            save_sync_settings,
            get_llm_config,
            save_llm_config,
            test_llm_connection,
            summarize_entry
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use crate::core::storage::models::EntryRecord;

    use super::build_summary_input;
    use super::fallback_entry_text;
    use super::hash_llm_input;
    use super::normalize_sync_settings;
    use super::parse_import_sources;
    use super::ImportRequest;
    use super::SyncSettings;

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

    #[test]
    fn sync_settings_are_normalized_to_safe_bounds() {
        let normalized = normalize_sync_settings(SyncSettings {
            interval_secs: 1,
            max_concurrency: 100,
            batch_limit: 9999,
            timeout_secs: 1,
            retry_count: 99,
        });

        assert_eq!(normalized.interval_secs, 60);
        assert_eq!(normalized.max_concurrency, 16);
        assert_eq!(normalized.batch_limit, 200);
        assert_eq!(normalized.timeout_secs, 5);
        assert_eq!(normalized.retry_count, 4);
    }

    #[test]
    fn fallback_entry_text_prefers_summary_and_content() {
        let entry = EntryRecord {
            id: 1,
            source_id: 1,
            source_title: "source".to_string(),
            guid: None,
            link: "https://example.com/post".to_string(),
            title: "Post title".to_string(),
            translated_title: None,
            summary: Some("summary".to_string()),
            content: Some("content".to_string()),
            published_at: None,
            is_read: 0,
            is_starred: 0,
            created_at: "2026-02-24T00:00:00Z".to_string(),
        };
        assert_eq!(fallback_entry_text(&entry), "summary\n\ncontent");
    }

    #[test]
    fn build_summary_input_is_capped() {
        let entry = EntryRecord {
            id: 1,
            source_id: 1,
            source_title: "source".to_string(),
            guid: None,
            link: "https://example.com/post".to_string(),
            title: "Post title".to_string(),
            translated_title: None,
            summary: None,
            content: None,
            published_at: None,
            is_read: 0,
            is_starred: 0,
            created_at: "2026-02-24T00:00:00Z".to_string(),
        };
        let huge = "a".repeat(13000);
        let input = build_summary_input(&entry, &huge);
        assert!(input.starts_with("Title: Post title"));
        assert!(input.contains("Article Text:"));
        assert!(input.len() < 12200);
    }
}
