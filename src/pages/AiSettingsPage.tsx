import { FormEvent, useEffect, useState } from "react";

import {
  getLlmConfig,
  getAppHealth,
  getSyncRuntimeStatus,
  getSyncSettings,
  isTauriRuntime,
  saveLlmConfig,
  saveSyncSettings,
  syncActiveSources,
  testLlmConnection,
  type HealthReport,
  type LlmConfig,
  type SyncRuntimeStatus,
  type SyncSettings,
} from "../services/tauriApi";

const DEFAULT_LLM_CONFIG: LlmConfig = {
  base_url: "https://api.deepseek.com/v1",
  api_key: "",
  model: "deepseek-chat",
  timeout_secs: 30,
};

const DEFAULT_SYNC_SETTINGS: SyncSettings = {
  interval_secs: 600,
  max_concurrency: 6,
  batch_limit: 24,
  timeout_secs: 12,
  retry_count: 1,
};

export function AiSettingsPage() {
  const [llmConfig, setLlmConfig] = useState<LlmConfig>(DEFAULT_LLM_CONFIG);
  const [syncSettings, setSyncSettings] = useState<SyncSettings>(DEFAULT_SYNC_SETTINGS);
  const [syncStatus, setSyncStatus] = useState<SyncRuntimeStatus | null>(null);
  const [health, setHealth] = useState<HealthReport>({});
  const [testResult, setTestResult] = useState("");
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);

  const canOperate = isTauriRuntime();

  async function refreshRuntimeStatus() {
    if (!canOperate) {
      return;
    }
    try {
      const status = await getSyncRuntimeStatus();
      setSyncStatus(status);
    } catch (err) {
      setError(err instanceof Error ? err.message : "获取同步状态失败");
    }
  }

  useEffect(() => {
    if (!canOperate) {
      return;
    }
    Promise.all([getLlmConfig(), getSyncSettings(), getSyncRuntimeStatus(), getAppHealth()])
      .then(([savedLlm, savedSync, runtime, healthReport]) => {
        if (savedLlm) {
          setLlmConfig(savedLlm);
        }
        setSyncSettings(savedSync);
        setSyncStatus(runtime);
        setHealth(healthReport);
      })
      .catch((err) => {
        setError(err instanceof Error ? err.message : "加载设置失败");
      });

    const timer = window.setInterval(() => {
      refreshRuntimeStatus();
    }, 2500);
    return () => {
      window.clearInterval(timer);
    };
  }, []);

  async function onSaveLlm(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canOperate) {
      return;
    }
    setSaving(true);
    setError("");
    setTestResult("");
    try {
      await saveLlmConfig(llmConfig);
      setTestResult("LLM 配置已保存");
    } catch (err) {
      setError(err instanceof Error ? err.message : "保存 LLM 配置失败");
    } finally {
      setSaving(false);
    }
  }

  async function onSaveSync(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canOperate) {
      return;
    }
    setSaving(true);
    setError("");
    try {
      const normalized = await saveSyncSettings(syncSettings);
      setSyncSettings(normalized);
      setTestResult("同步配置已保存");
    } catch (err) {
      setError(err instanceof Error ? err.message : "保存同步配置失败");
    } finally {
      setSaving(false);
    }
  }

  async function onTestLlm() {
    if (!canOperate) {
      return;
    }
    setError("");
    setTestResult("");
    try {
      const result = await testLlmConnection(llmConfig);
      setTestResult(`连通性测试成功：${result}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "连通性测试失败");
    }
  }

  async function onManualSync() {
    if (!canOperate) {
      return;
    }
    setError("");
    try {
      const status = await syncActiveSources();
      setSyncStatus(status);
    } catch (err) {
      setError(err instanceof Error ? err.message : "启动同步失败");
    }
  }

  if (!canOperate) {
    return (
      <section className="page-grid">
        <article className="page-card">
          <h2>设置</h2>
          <p>当前是浏览器测试环境，Tauri 命令不可用。请在桌面端运行查看完整功能。</p>
        </article>
      </section>
    );
  }

  return (
    <section className="page-grid">
      <article className="page-card">
        <h2>系统状态</h2>
        {Object.entries(health).map(([name, status]) => (
          <p key={name}>
            {name}: {status}
          </p>
        ))}
      </article>

      <article className="page-card">
        <h2>同步状态</h2>
        <p>{syncStatus?.running ? "同步进行中..." : "同步空闲"}</p>
        {syncStatus?.last_report ? (
          <p>
            最近一次：成功 {syncStatus.last_report.synced_sources}，失败{" "}
            {syncStatus.last_report.failed_sources}，写入文章{" "}
            {syncStatus.last_report.total_upserted_entries}
          </p>
        ) : null}
        {syncStatus?.last_error ? <p className="inline-error">{syncStatus.last_error}</p> : null}
        <div className="button-row">
          <button type="button" onClick={onManualSync}>
            手动触发同步
          </button>
          <button type="button" onClick={refreshRuntimeStatus}>
            刷新状态
          </button>
        </div>
      </article>

      <article className="page-card">
        <h2>同步参数</h2>
        <form className="form-grid" onSubmit={onSaveSync}>
          <input
            type="number"
            value={syncSettings.interval_secs}
            onChange={(event) =>
              setSyncSettings((current) => ({
                ...current,
                interval_secs: Number(event.target.value || 600),
              }))
            }
            placeholder="同步间隔（秒）"
          />
          <input
            type="number"
            value={syncSettings.max_concurrency}
            onChange={(event) =>
              setSyncSettings((current) => ({
                ...current,
                max_concurrency: Number(event.target.value || 6),
              }))
            }
            placeholder="并发数"
          />
          <input
            type="number"
            value={syncSettings.batch_limit}
            onChange={(event) =>
              setSyncSettings((current) => ({
                ...current,
                batch_limit: Number(event.target.value || 24),
              }))
            }
            placeholder="每轮最大源数"
          />
          <input
            type="number"
            value={syncSettings.timeout_secs}
            onChange={(event) =>
              setSyncSettings((current) => ({
                ...current,
                timeout_secs: Number(event.target.value || 12),
              }))
            }
            placeholder="请求超时（秒）"
          />
          <input
            type="number"
            value={syncSettings.retry_count}
            onChange={(event) =>
              setSyncSettings((current) => ({
                ...current,
                retry_count: Number(event.target.value || 1),
              }))
            }
            placeholder="重试次数"
          />
          <button type="submit" disabled={saving}>
            保存同步配置
          </button>
        </form>
      </article>

      <article className="page-card page-wide">
        <h2>LLM Provider（OpenAI Compatible）</h2>
        <form className="form-grid" onSubmit={onSaveLlm}>
          <input
            placeholder="Base URL"
            value={llmConfig.base_url}
            onChange={(event) =>
              setLlmConfig((current) => ({ ...current, base_url: event.target.value }))
            }
          />
          <input
            placeholder="API Key"
            value={llmConfig.api_key}
            onChange={(event) =>
              setLlmConfig((current) => ({ ...current, api_key: event.target.value }))
            }
          />
          <input
            placeholder="Model"
            value={llmConfig.model}
            onChange={(event) =>
              setLlmConfig((current) => ({ ...current, model: event.target.value }))
            }
          />
          <input
            placeholder="超时时间（秒）"
            type="number"
            value={llmConfig.timeout_secs}
            onChange={(event) =>
              setLlmConfig((current) => ({
                ...current,
                timeout_secs: Number(event.target.value || 30),
              }))
            }
          />
          <div className="button-row">
            <button type="submit" disabled={saving}>
              保存 LLM 配置
            </button>
            <button type="button" onClick={onTestLlm}>
              测试连通性
            </button>
          </div>
        </form>
        {testResult ? <p>{testResult}</p> : null}
        {error ? <p className="inline-error">{error}</p> : null}
      </article>
    </section>
  );
}
