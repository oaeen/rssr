import { invoke } from "@tauri-apps/api/core";

export type HealthReport = Record<string, string>;
export type Source = {
  id: number;
  title: string;
  site_url: string | null;
  feed_url: string;
  category: string | null;
  is_active: boolean;
  failure_count: number;
  created_at: string;
  updated_at: string;
};

export type UpsertSourceRequest = {
  title: string;
  site_url?: string | null;
  feed_url: string;
  category?: string | null;
  is_active: boolean;
};

export type ImportRequest = {
  format: "opml" | "xml" | "url_list" | "json";
  content: string;
  default_category?: string;
  is_active?: boolean;
};

export type ImportPreviewResponse = {
  new_count: number;
  duplicate_count: number;
  new_sources: Array<{
    title: string;
    feed_url: string;
    site_url: string | null;
    category: string | null;
  }>;
  duplicate_sources: Array<{
    title: string;
    feed_url: string;
    site_url: string | null;
    category: string | null;
  }>;
};

export type ImportExecuteResponse = {
  imported_count: number;
  duplicate_count: number;
};

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && typeof window.__TAURI_INTERNALS__ !== "undefined";
}

export async function getAppHealth(): Promise<HealthReport> {
  return invoke<HealthReport>("app_health");
}

export async function listSources(): Promise<Source[]> {
  return invoke<Source[]>("list_sources");
}

export async function upsertSource(payload: UpsertSourceRequest): Promise<Source> {
  return invoke<Source>("upsert_source", { request: payload });
}

export async function deleteSource(id: number): Promise<number> {
  return invoke<number>("delete_source", { id });
}

export async function setSourcesActive(sourceIds: number[], isActive: boolean): Promise<number> {
  return invoke<number>("set_sources_active", { sourceIds, isActive });
}

export async function previewImport(payload: ImportRequest): Promise<ImportPreviewResponse> {
  return invoke<ImportPreviewResponse>("preview_import", { request: payload });
}

export async function importSources(payload: ImportRequest): Promise<ImportExecuteResponse> {
  return invoke<ImportExecuteResponse>("import_sources", { request: payload });
}
