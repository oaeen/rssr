import { useMemo, useState } from "react";

import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

import { ImportPage } from "../pages/ImportPage";
import { ReaderPage } from "../pages/ReaderPage";
import { SubscriptionsPage } from "../pages/SubscriptionsPage";
import { isTauriRuntime } from "../services/tauriApi";
import type { AppTab } from "../store";

const TABS: Array<{ id: AppTab; label: string }> = [
  { id: "reader", label: "文章" },
  { id: "subscriptions", label: "订阅" },
  { id: "import", label: "导入" },
];

export function AppShell() {
  const [activeTab, setActiveTab] = useState<AppTab>("reader");
  const [error, setError] = useState("");

  const page = useMemo(() => {
    if (activeTab === "reader") {
      return <ReaderPage />;
    }
    if (activeTab === "subscriptions") {
      return <SubscriptionsPage />;
    }
    return <ImportPage />;
  }, [activeTab]);

  async function openSettingsWindow() {
    if (!isTauriRuntime()) {
      setError("设置窗口仅在桌面端可用");
      return;
    }

    const label = "settings-window";
    const existed = await WebviewWindow.getByLabel(label);
    if (existed) {
      await existed.setFocus();
      return;
    }

    const win = new WebviewWindow(label, {
      title: "RSSR 设置",
      width: 980,
      height: 760,
      minWidth: 860,
      minHeight: 640,
      center: true,
      url: "index.html?window=settings",
    });
    win.once("tauri://error", (event) => {
      setError(String(event.payload));
    });
  }

  return (
    <main className="workspace-shell" data-testid="app-shell">
      <header className="workspace-topbar">
        <div className="workspace-brand">
          <h1 className="app-title">RSSR</h1>
          <p className="tiny-muted">Folo 风格阅读工作台</p>
        </div>
        <nav className="tabs" aria-label="导航标签">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              className={tab.id === activeTab ? "tab tab-active" : "tab"}
              onClick={() => setActiveTab(tab.id)}
              type="button"
            >
              {tab.label}
            </button>
          ))}
        </nav>
        <div className="workspace-actions">
          <button type="button" onClick={openSettingsWindow}>
            设置
          </button>
        </div>
      </header>
      {error ? <p className="inline-error">{error}</p> : null}
      <section className="workspace-content">{page}</section>
    </main>
  );
}
