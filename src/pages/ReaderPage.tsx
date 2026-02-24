import { useEffect, useMemo, useState } from "react";

import {
  isTauriRuntime,
  listEntries,
  markEntryRead,
  syncActiveSources,
  type Entry,
} from "../services/tauriApi";

export function ReaderPage() {
  const [entries, setEntries] = useState<Entry[]>([]);
  const [activeEntryId, setActiveEntryId] = useState<number | null>(null);
  const [search, setSearch] = useState("");
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [loading, setLoading] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  const canOperate = isTauriRuntime();

  const activeEntry = useMemo(
    () => entries.find((entry) => entry.id === activeEntryId) ?? null,
    [entries, activeEntryId],
  );

  async function refreshEntries() {
    if (!canOperate) {
      return;
    }
    setLoading(true);
    setError("");
    try {
      const items = await listEntries({
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

  useEffect(() => {
    refreshEntries();
  }, [search, unreadOnly]);

  async function onSyncActiveSources() {
    if (!canOperate) {
      return;
    }
    setSyncing(true);
    setError("");
    setMessage("");
    try {
      const result = await syncActiveSources();
      setMessage(
        `同步完成：成功 ${result.synced_sources}，失败 ${result.failed_sources}，新增/更新文章 ${result.total_upserted_entries}`,
      );
      await refreshEntries();
    } catch (err) {
      setError(err instanceof Error ? err.message : "同步失败");
    } finally {
      setSyncing(false);
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
    <section className="reader-layout">
      <article className="page-card reader-list-panel">
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
            <span>仅看未读</span>
          </label>
          <div className="button-row">
            <button onClick={refreshEntries} type="button">
              刷新
            </button>
            <button onClick={onSyncActiveSources} type="button" disabled={syncing}>
              {syncing ? "同步中..." : "同步订阅"}
            </button>
          </div>
        </div>
        {message ? <p>{message}</p> : null}
        {error ? <p className="inline-error">{error}</p> : null}
        {loading ? <p>加载中...</p> : null}
        {!loading && entries.length === 0 ? <p>暂无文章，先去“订阅”页添加并同步。</p> : null}
        <div className="reader-list">
          {entries.map((entry) => (
            <button
              className={entry.id === activeEntryId ? "reader-item reader-item-active" : "reader-item"}
              key={entry.id}
              onClick={() => setActiveEntryId(entry.id)}
              type="button"
            >
              <strong>{entry.title}</strong>
              <span>{entry.source_title}</span>
              <span>{entry.is_read ? "已读" : "未读"}</span>
            </button>
          ))}
        </div>
      </article>

      <article className="page-card reader-detail-panel">
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
              <a href={activeEntry.link} target="_blank" rel="noreferrer">
                打开原文
              </a>
            </div>
            <article className="reader-content">
              {activeEntry.content ? (
                <pre>{activeEntry.content}</pre>
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
