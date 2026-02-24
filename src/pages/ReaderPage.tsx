import { useEffect, useMemo, useState } from "react";

import {
  getSyncRuntimeStatus,
  isTauriRuntime,
  listEntries,
  listSources,
  markEntryRead,
  summarizeEntry,
  syncActiveSources,
  translateEntry,
  type Entry,
  type Source,
  type SyncRuntimeStatus,
} from "../services/tauriApi";

export function ReaderPage() {
  const [sources, setSources] = useState<Source[]>([]);
  const [entries, setEntries] = useState<Entry[]>([]);
  const [selectedSourceId, setSelectedSourceId] = useState<number | undefined>(undefined);
  const [activeEntryId, setActiveEntryId] = useState<number | null>(null);
  const [search, setSearch] = useState("");
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [loading, setLoading] = useState(false);
  const [syncStatus, setSyncStatus] = useState<SyncRuntimeStatus | null>(null);
  const [summaryResult, setSummaryResult] = useState("");
  const [translationResult, setTranslationResult] = useState("");
  const [aiLoading, setAiLoading] = useState(false);
  const [error, setError] = useState("");

  const canOperate = isTauriRuntime();

  const activeEntry = useMemo(
    () => entries.find((entry) => entry.id === activeEntryId) ?? null,
    [entries, activeEntryId],
  );

  async function refreshSources() {
    if (!canOperate) {
      return;
    }
    try {
      const response = await listSources();
      setSources(response);
    } catch (err) {
      setError(err instanceof Error ? err.message : "加载订阅源失败");
    }
  }

  async function refreshEntries() {
    if (!canOperate) {
      return;
    }
    setLoading(true);
    setError("");
    try {
      const items = await listEntries({
        source_id: selectedSourceId,
        search: search.trim() || undefined,
        unread_only: unreadOnly,
        limit: 300,
      });
      setEntries(items);
      if (items.length > 0 && !activeEntryId) {
        setActiveEntryId(items[0].id);
      }
      if (items.length === 0) {
        setActiveEntryId(null);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "加载文章失败");
    } finally {
      setLoading(false);
    }
  }

  async function refreshSyncStatus() {
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
    refreshSources();
    refreshEntries();
    refreshSyncStatus();
  }, []);

  useEffect(() => {
    refreshEntries();
  }, [selectedSourceId, search, unreadOnly]);

  useEffect(() => {
    if (!canOperate) {
      return;
    }
    const timer = window.setInterval(() => {
      refreshSyncStatus();
    }, 2500);
    return () => {
      window.clearInterval(timer);
    };
  }, []);

  async function onStartSync() {
    if (!canOperate) {
      return;
    }
    setError("");
    try {
      const status = await syncActiveSources();
      setSyncStatus(status);
    } catch (err) {
      setError(err instanceof Error ? err.message : "触发同步失败");
    }
  }

  async function onMarkRead(entry: Entry, isRead: boolean) {
    if (!canOperate) {
      return;
    }
    setError("");
    try {
      await markEntryRead(entry.id, isRead);
      await refreshEntries();
    } catch (err) {
      setError(err instanceof Error ? err.message : "更新已读状态失败");
    }
  }

  async function onSummarize() {
    if (!canOperate || !activeEntry) {
      return;
    }
    setAiLoading(true);
    setError("");
    try {
      const result = await summarizeEntry(activeEntry.id);
      setSummaryResult(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "AI 总结失败");
    } finally {
      setAiLoading(false);
    }
  }

  async function onTranslate() {
    if (!canOperate || !activeEntry) {
      return;
    }
    setAiLoading(true);
    setError("");
    try {
      const result = await translateEntry(activeEntry.id, "Chinese");
      setTranslationResult(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "AI 翻译失败");
    } finally {
      setAiLoading(false);
    }
  }

  if (!canOperate) {
    return (
      <section className="page-grid">
        <article className="page-card">
          <h2>阅读面板</h2>
          <p>当前是浏览器测试环境，Tauri 命令不可用。请在桌面端运行查看完整功能。</p>
        </article>
      </section>
    );
  }

  return (
    <section className="folo-layout">
      <aside className="folo-sidebar">
        <div className="folo-toolbar">
          <button type="button" onClick={onStartSync}>
            {syncStatus?.running ? "同步中..." : "同步"}
          </button>
          <button type="button" onClick={refreshEntries}>
            刷新
          </button>
        </div>
        <p className="tiny-muted">
          {syncStatus?.last_report
            ? `上次写入 ${syncStatus.last_report.total_upserted_entries} 篇`
            : "暂无同步记录"}
        </p>
        <div className="source-tree">
          <button
            type="button"
            className={selectedSourceId === undefined ? "source-item source-item-active" : "source-item"}
            onClick={() => setSelectedSourceId(undefined)}
          >
            全部文章
          </button>
          {sources.map((source) => (
            <button
              key={source.id}
              type="button"
              className={selectedSourceId === source.id ? "source-item source-item-active" : "source-item"}
              onClick={() => setSelectedSourceId(source.id)}
            >
              <span>{source.title}</span>
              <span className="tiny-muted">{source.failure_count > 0 ? `失败 ${source.failure_count}` : ""}</span>
            </button>
          ))}
        </div>
      </aside>

      <section className="folo-list-panel">
        <div className="reader-toolbar">
          <input
            placeholder="搜索标题或摘要"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
          />
          <label className="checkbox-line">
            <input
              checked={unreadOnly}
              onChange={(event) => setUnreadOnly(event.target.checked)}
              type="checkbox"
            />
            <span>仅未读</span>
          </label>
        </div>
        {loading ? <p>加载中...</p> : null}
        {!loading && entries.length === 0 ? <p>暂无文章，先去订阅并同步。</p> : null}
        <div className="reader-list">
          {entries.map((entry) => (
            <button
              className={entry.id === activeEntryId ? "reader-item reader-item-active" : "reader-item"}
              key={entry.id}
              onClick={() => {
                setActiveEntryId(entry.id);
                setSummaryResult("");
                setTranslationResult("");
              }}
              type="button"
            >
              <strong>{entry.title}</strong>
              <span>{entry.source_title}</span>
              <span>{entry.is_read ? "已读" : "未读"}</span>
            </button>
          ))}
        </div>
      </section>

      <article className="folo-detail-panel">
        {error ? <p className="inline-error">{error}</p> : null}
        {!activeEntry ? (
          <p>选择左侧文章开始阅读。</p>
        ) : (
          <>
            <h2>{activeEntry.title}</h2>
            <p>{activeEntry.summary ?? "无摘要"}</p>
            <div className="button-row">
              <button onClick={() => onMarkRead(activeEntry, !activeEntry.is_read)} type="button">
                {activeEntry.is_read ? "标记未读" : "标记已读"}
              </button>
              <button onClick={onSummarize} disabled={aiLoading} type="button">
                {aiLoading ? "处理中..." : "AI 总结"}
              </button>
              <button onClick={onTranslate} disabled={aiLoading} type="button">
                {aiLoading ? "处理中..." : "翻译为中文"}
              </button>
              <a href={activeEntry.link} target="_blank" rel="noreferrer">
                原文
              </a>
            </div>
            {(summaryResult || translationResult) && (
              <section className="ai-card">
                {summaryResult ? (
                  <>
                    <h3>AI 总结</h3>
                    <pre>{summaryResult}</pre>
                  </>
                ) : null}
                {translationResult ? (
                  <>
                    <h3>AI 翻译</h3>
                    <pre>{translationResult}</pre>
                  </>
                ) : null}
              </section>
            )}
            <article className="reader-content">
              {activeEntry.content ? <pre>{activeEntry.content}</pre> : <p>暂无正文，查看原文链接。</p>}
            </article>
          </>
        )}
      </article>
    </section>
  );
}
