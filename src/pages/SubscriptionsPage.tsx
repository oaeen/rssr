import { FormEvent, useEffect, useMemo, useState } from "react";

import {
  deleteSource,
  importSources,
  isTauriRuntime,
  listSources,
  previewImport,
  setSourcesActive,
  type ImportPreviewResponse,
  type Source,
  upsertSource,
} from "../services/tauriApi";

type FeedFormState = {
  title: string;
  feedUrl: string;
  siteUrl: string;
  category: string;
};

const DEFAULT_FORM: FeedFormState = {
  title: "",
  feedUrl: "",
  siteUrl: "",
  category: "",
};

export function SubscriptionsPage() {
  const [sources, setSources] = useState<Source[]>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [form, setForm] = useState<FeedFormState>(DEFAULT_FORM);
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [importFormat, setImportFormat] = useState<"opml" | "xml" | "url_list" | "json">("opml");
  const [importContent, setImportContent] = useState("");
  const [importPreviewResult, setImportPreviewResult] = useState<ImportPreviewResponse | null>(null);
  const [importMessage, setImportMessage] = useState("");

  const canOperate = isTauriRuntime();

  const selectedCount = useMemo(() => selectedIds.length, [selectedIds]);

  async function refreshSources() {
    if (!canOperate) {
      return;
    }
    setLoading(true);
    setError("");
    try {
      const response = await listSources();
      setSources(response);
    } catch (err) {
      setError(err instanceof Error ? err.message : "加载订阅失败");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    refreshSources();
  }, []);

  async function onAddSource(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canOperate) {
      return;
    }
    if (!form.feedUrl.trim()) {
      setError("feed URL 不能为空");
      return;
    }
    setSaving(true);
    setError("");
    try {
      await upsertSource({
        title: form.title.trim() || form.feedUrl.trim(),
        feed_url: form.feedUrl.trim(),
        site_url: form.siteUrl.trim() || null,
        category: form.category.trim() || null,
        is_active: true,
      });
      setForm(DEFAULT_FORM);
      await refreshSources();
    } catch (err) {
      setError(err instanceof Error ? err.message : "新增订阅失败");
    } finally {
      setSaving(false);
    }
  }

  async function onDeleteSource(id: number) {
    if (!canOperate) {
      return;
    }
    setError("");
    try {
      await deleteSource(id);
      setSelectedIds((current) => current.filter((value) => value !== id));
      await refreshSources();
    } catch (err) {
      setError(err instanceof Error ? err.message : "删除失败");
    }
  }

  async function onBatchSetActive(isActive: boolean) {
    if (!canOperate || selectedIds.length === 0) {
      return;
    }
    setError("");
    try {
      await setSourcesActive(selectedIds, isActive);
      await refreshSources();
    } catch (err) {
      setError(err instanceof Error ? err.message : "批量更新失败");
    }
  }

  async function onPreviewImport() {
    if (!canOperate || !importContent.trim()) {
      return;
    }
    setError("");
    setImportMessage("");
    try {
      const preview = await previewImport({
        format: importFormat,
        content: importContent,
      });
      setImportPreviewResult(preview);
    } catch (err) {
      setError(err instanceof Error ? err.message : "导入预览失败");
    }
  }

  async function onExecuteImport() {
    if (!canOperate || !importContent.trim()) {
      return;
    }
    setError("");
    try {
      const result = await importSources({
        format: importFormat,
        content: importContent,
        is_active: true,
      });
      setImportMessage(
        `导入完成：新增 ${result.imported_count}，跳过重复 ${result.duplicate_count}`,
      );
      setImportPreviewResult(null);
      await refreshSources();
    } catch (err) {
      setError(err instanceof Error ? err.message : "导入失败");
    }
  }

  function toggleSelected(id: number) {
    setSelectedIds((current) => {
      if (current.includes(id)) {
        return current.filter((value) => value !== id);
      }
      return [...current, id];
    });
  }

  if (!canOperate) {
    return (
      <section className="page-grid">
        <article className="page-card">
          <h2>订阅管理</h2>
          <p>当前是浏览器测试环境，Tauri 命令不可用。请在桌面端运行查看完整功能。</p>
        </article>
      </section>
    );
  }

  return (
    <section className="page-grid">
      <article className="page-card">
        <h2>新增订阅</h2>
        <form className="form-grid" onSubmit={onAddSource}>
          <input
            placeholder="标题（可选）"
            value={form.title}
            onChange={(event) => setForm((current) => ({ ...current, title: event.target.value }))}
          />
          <input
            placeholder="Feed URL"
            value={form.feedUrl}
            onChange={(event) =>
              setForm((current) => ({ ...current, feedUrl: event.target.value }))
            }
          />
          <input
            placeholder="站点 URL（可选）"
            value={form.siteUrl}
            onChange={(event) =>
              setForm((current) => ({ ...current, siteUrl: event.target.value }))
            }
          />
          <input
            placeholder="分组（可选）"
            value={form.category}
            onChange={(event) =>
              setForm((current) => ({ ...current, category: event.target.value }))
            }
          />
          <button disabled={saving} type="submit">
            {saving ? "保存中..." : "新增订阅"}
          </button>
        </form>
      </article>

      <article className="page-card">
        <h2>批量导入</h2>
        <div className="form-grid">
          <select
            value={importFormat}
            onChange={(event) =>
              setImportFormat(event.target.value as "opml" | "xml" | "url_list" | "json")
            }
          >
            <option value="opml">OPML/XML</option>
            <option value="url_list">URL 列表</option>
            <option value="json">JSON</option>
          </select>
          <textarea
            rows={8}
            placeholder="粘贴 OPML、URL 列表或 JSON 内容"
            value={importContent}
            onChange={(event) => setImportContent(event.target.value)}
          />
          <div className="button-row">
            <button onClick={onPreviewImport} type="button">
              预览导入
            </button>
            <button onClick={onExecuteImport} type="button">
              执行导入
            </button>
          </div>
          {importPreviewResult ? (
            <p>
              预览结果：新增 {importPreviewResult.new_count}，重复{" "}
              {importPreviewResult.duplicate_count}
            </p>
          ) : null}
          {importMessage ? <p>{importMessage}</p> : null}
        </div>
      </article>

      <article className="page-card page-wide">
        <div className="subscriptions-head">
          <h2>订阅列表</h2>
          <div className="button-row">
            <button onClick={() => onBatchSetActive(true)} type="button">
              批量启用（{selectedCount}）
            </button>
            <button onClick={() => onBatchSetActive(false)} type="button">
              批量停用（{selectedCount}）
            </button>
            <button onClick={refreshSources} type="button">
              刷新
            </button>
          </div>
        </div>
        {error ? <p className="inline-error">{error}</p> : null}
        {loading ? <p>加载中...</p> : null}
        {!loading && sources.length === 0 ? <p>暂无订阅</p> : null}
        {sources.map((source) => (
          <div className="source-row" key={source.id}>
            <label className="checkbox-line">
              <input
                checked={selectedIds.includes(source.id)}
                onChange={() => toggleSelected(source.id)}
                type="checkbox"
              />
              <span>{source.title}</span>
            </label>
            <p className="source-url">{source.feed_url}</p>
            <div className="button-row">
              <span className={source.is_active ? "tag tag-on" : "tag tag-off"}>
                {source.is_active ? "已启用" : "已停用"}
              </span>
              <button onClick={() => onDeleteSource(source.id)} type="button">
                删除
              </button>
            </div>
          </div>
        ))}
      </article>
    </section>
  );
}
