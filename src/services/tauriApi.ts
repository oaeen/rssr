import { invoke } from "@tauri-apps/api/core";

export type HealthReport = Record<string, string>;

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
