import { useEffect, useMemo, useState } from "react";

import {
  getSyncRuntimeStatus,
  isTauriRuntime,
  listEntries,
  listSources,
  markEntryRead,
  summarizeEntry,
  syncActiveSources,
  type Entry,
  type Source,
  type SyncRuntimeStatus,
} from "../services/tauriApi";
import { toPlainText, toSafeHtml } from "../utils/richText";

const ENTRY_TIME_FORMATTER = new Intl.DateTimeFormat("zh-CN", {
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
});

export function resolveDisplayTitles(
  title: string | null | undefined,
  translatedTitle: string | null | undefined,
): { primary: string; secondary: string | null } {
  const original = toPlainText(title) || "Untitled";
  const translated = toPlainText(translatedTitle);
  if (!translated || translated === original) {
    return { primary: original, secondary: null };
  }
  return { primary: translated, secondary: original };
}

export function formatEntryTime(value: string | null | undefined): string {
  if (!value) {
    return "未知时间";
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "未知时间";
  }
  return ENTRY_TIME_FORMATTER.format(date);
}

export function buildSourceIconUrl(source: Source): string {
  const candidate = source.site_url?.trim() || source.feed_url.trim();
  try {
    const parsed = new URL(candidate);
    return `https://www.google.com/s2/favicons?domain=${encodeURIComponent(parsed.hostname)}&sz=64`;
  } catch {
    return "";
  }
}

type ReaderPageProps = {
  onOpenSettings?: () => void;
};

export function ReaderPage({ onOpenSettings }: ReaderPageProps = {}) {
  const [sources, setSources] = useState<Source[]>([]);
  const [entries, setEntries] = useState<Entry[]>([]);
  const [selectedSourceId, setSelectedSourceId] = useState<number | undefined>(undefined);
  const [activeEntryId, setActiveEntryId] = useState<number | null>(null);
  const [search, setSearch] = useState("");
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [loading, setLoading] = useState(false);
  const [syncStatus, setSyncStatus] = useState<SyncRuntimeStatus | null>(null);
  const [summaryResult, setSummaryResult] = useState("");
  const [aiLoading, setAiLoading] = useState(false);
  const [error, setError] = useState("");

  const canOperate = isTauriRuntime();

  const activeEntry = useMemo(
    () => entries.find((entry) => entry.id === activeEntryId) ?? null,
    [entries, activeEntryId],
  );
  const activeTitle = useMemo(
    () =>
      activeEntry ? resolveDisplayTitles(activeEntry.title, activeEntry.translated_title) : null,
    [activeEntry],
  );
  const summaryHtml = useMemo(() => toSafeHtml(activeEntry?.summary), [activeEntry?.summary]);
  const contentHtml = useMemo(() => toSafeHtml(activeEntry?.content), [activeEntry?.content]);

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
          {onOpenSettings ? (
            <button type="button" onClick={onOpenSettings}>
              设置
            </button>
          ) : null}
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
          {sources.map((source) => {
            const iconUrl = buildSourceIconUrl(source);
            const titleText = source.title.trim() || source.feed_url;
            return (
              <button
                key={source.id}
                type="button"
                className={selectedSourceId === source.id ? "source-item source-item-active" : "source-item"}
                onClick={() => setSelectedSourceId(source.id)}
              >
                <span className="source-item-main">
                  {iconUrl ? (
                    <img className="source-icon" src={iconUrl} alt="" loading="lazy" />
                  ) : (
                    <span className="source-icon source-icon-fallback" aria-hidden="true">
                      •
                    </span>
                  )}
                  <span className="source-item-title" title={titleText}>
                    {titleText}
                  </span>
                </span>
                <span className="tiny-muted">{source.failure_count > 0 ? `失败 ${source.failure_count}` : ""}</span>
              </button>
            );
          })}
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
          {entries.map((entry) => {
            const title = resolveDisplayTitles(entry.title, entry.translated_title);
            return (
              <button
                className={entry.id === activeEntryId ? "reader-item reader-item-active" : "reader-item"}
                key={entry.id}
                onClick={() => {
                  setActiveEntryId(entry.id);
                  setSummaryResult("");
                }}
                type="button"
              >
                <strong>{title.primary}</strong>
                {title.secondary ? <span className="reader-item-subtitle">{title.secondary}</span> : null}
                <span>{entry.source_title}</span>
                <span className="reader-item-meta">
                  {formatEntryTime(entry.published_at ?? entry.created_at)} · {entry.is_read ? "已读" : "未读"}
                </span>
              </button>
            );
          })}
        </div>
      </section>

      <article className="folo-detail-panel">
        {error ? <p className="inline-error">{error}</p> : null}
        {!activeEntry ? (
          <p>选择左侧文章开始阅读。</p>
        ) : (
          <>
            <header className="article-head">
              <div className="article-head-main">
                <h2>{activeTitle?.primary ?? "Untitled"}</h2>
                {activeTitle?.secondary ? <p className="article-meta">原题：{activeTitle.secondary}</p> : null}
                <p className="article-meta">发布时间：{formatEntryTime(activeEntry.published_at ?? activeEntry.created_at)}</p>
              </div>
              <div className="article-actions">
                <button
                  className="icon-action"
                  onClick={() => onMarkRead(activeEntry, !activeEntry.is_read)}
                  type="button"
                  title={activeEntry.is_read ? "标记未读" : "标记已读"}
                  aria-label={activeEntry.is_read ? "标记未读" : "标记已读"}
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <path d="M9 12l2 2 4-4" />
                    <path d="M12 3a9 9 0 100 18 9 9 0 000-18z" />
                  </svg>
                </button>
                <button
                  className="icon-action"
                  onClick={onSummarize}
                  disabled={aiLoading}
                  type="button"
                  title={aiLoading ? "AI 总结处理中" : "AI 总结"}
                  aria-label="AI 总结"
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <path d="M12 3l1.8 4.2L18 9l-4.2 1.8L12 15l-1.8-4.2L6 9l4.2-1.8L12 3z" />
                  </svg>
                </button>
                <a
                  className="icon-action"
                  href={activeEntry.link}
                  target="_blank"
                  rel="noreferrer"
                  title="打开原文"
                  aria-label="打开原文"
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <path d="M14 4h6v6" />
                    <path d="M10 14L20 4" />
                    <path d="M20 14v6H4V4h6" />
                  </svg>
                </a>
              </div>
            </header>
            {summaryHtml ? (
              <div className="article-summary article-html" dangerouslySetInnerHTML={{ __html: summaryHtml }} />
            ) : (
              <p>无摘要</p>
            )}
            {summaryResult && (
              <section className="ai-card">
                <h3>AI 总结</h3>
                <pre>{summaryResult}</pre>
              </section>
            )}
            <article className="reader-content">
              {contentHtml ? (
                <div className="article-html" dangerouslySetInnerHTML={{ __html: contentHtml }} />
              ) : (
                <p>暂无正文，查看原文链接。</p>
              )}
            </article>
          </>
        )}
      </article>
    </section>
  );
}
