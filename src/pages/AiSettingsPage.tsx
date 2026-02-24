import { FormEvent, useEffect, useState } from "react";

import {
  getLlmConfig,
  isTauriRuntime,
  saveLlmConfig,
  summarizeEntry,
  testLlmConnection,
  translateEntry,
  type LlmConfig,
} from "../services/tauriApi";

const DEFAULT_CONFIG: LlmConfig = {
  base_url: "https://api.deepseek.com/v1",
  api_key: "",
  model: "deepseek-chat",
  timeout_secs: 30,
};

export function AiSettingsPage() {
  const [config, setConfig] = useState<LlmConfig>(DEFAULT_CONFIG);
  const [entryId, setEntryId] = useState("");
  const [targetLanguage, setTargetLanguage] = useState("Chinese");
  const [testResult, setTestResult] = useState("");
  const [summaryResult, setSummaryResult] = useState("");
  const [translationResult, setTranslationResult] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const canOperate = isTauriRuntime();

  useEffect(() => {
    if (!canOperate) {
      return;
    }
    getLlmConfig()
      .then((saved) => {
        if (saved) {
          setConfig(saved);
        }
      })
      .catch((err) => {
        setError(err instanceof Error ? err.message : "加载 LLM 配置失败");
      });
  }, []);

  async function onSaveConfig(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canOperate) {
      return;
    }
    setSaving(true);
    setError("");
    try {
      await saveLlmConfig(config);
      setTestResult("配置已保存");
    } catch (err) {
      setError(err instanceof Error ? err.message : "保存配置失败");
    } finally {
      setSaving(false);
    }
  }

  async function onTestConnection() {
    if (!canOperate) {
      return;
    }
    setError("");
    setTestResult("");
    try {
      const result = await testLlmConnection(config);
      setTestResult(`连通性测试成功：${result}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "连通性测试失败");
    }
  }

  async function onSummarize() {
    if (!canOperate || !entryId.trim()) {
      return;
    }
    setError("");
    setSummaryResult("");
    try {
      const result = await summarizeEntry(Number(entryId));
      setSummaryResult(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "总结失败");
    }
  }

  async function onTranslate() {
    if (!canOperate || !entryId.trim()) {
      return;
    }
    setError("");
    setTranslationResult("");
    try {
      const result = await translateEntry(Number(entryId), targetLanguage);
      setTranslationResult(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "翻译失败");
    }
  }

  if (!canOperate) {
    return (
      <section className="page-grid">
        <article className="page-card">
          <h2>AI Provider 设置</h2>
          <p>当前是浏览器测试环境，Tauri 命令不可用。请在桌面端运行查看完整功能。</p>
        </article>
      </section>
    );
  }

  return (
    <section className="page-grid">
      <article className="page-card">
        <h2>OpenAI Compatible 配置</h2>
        <form className="form-grid" onSubmit={onSaveConfig}>
          <input
            placeholder="Base URL"
            value={config.base_url}
            onChange={(event) =>
              setConfig((current) => ({ ...current, base_url: event.target.value }))
            }
          />
          <input
            placeholder="API Key"
            value={config.api_key}
            onChange={(event) =>
              setConfig((current) => ({ ...current, api_key: event.target.value }))
            }
          />
          <input
            placeholder="Model"
            value={config.model}
            onChange={(event) =>
              setConfig((current) => ({ ...current, model: event.target.value }))
            }
          />
          <input
            placeholder="超时时间（秒）"
            type="number"
            value={config.timeout_secs}
            onChange={(event) =>
              setConfig((current) => ({
                ...current,
                timeout_secs: Number(event.target.value || 30),
              }))
            }
          />
          <div className="button-row">
            <button type="submit" disabled={saving}>
              {saving ? "保存中..." : "保存配置"}
            </button>
            <button type="button" onClick={onTestConnection}>
              测试连通性
            </button>
          </div>
        </form>
        {testResult ? <p>{testResult}</p> : null}
        {error ? <p className="inline-error">{error}</p> : null}
      </article>

      <article className="page-card">
        <h2>翻译与总结</h2>
        <div className="form-grid">
          <input
            placeholder="文章 ID（来自阅读页）"
            value={entryId}
            onChange={(event) => setEntryId(event.target.value)}
          />
          <input
            placeholder="目标语言（例如 Chinese / Japanese）"
            value={targetLanguage}
            onChange={(event) => setTargetLanguage(event.target.value)}
          />
          <div className="button-row">
            <button onClick={onSummarize} type="button">
              生成总结
            </button>
            <button onClick={onTranslate} type="button">
              生成翻译
            </button>
          </div>
          {summaryResult ? (
            <article className="reader-content">
              <pre>{summaryResult}</pre>
            </article>
          ) : null}
          {translationResult ? (
            <article className="reader-content">
              <pre>{translationResult}</pre>
            </article>
          ) : null}
        </div>
      </article>
    </section>
  );
}
