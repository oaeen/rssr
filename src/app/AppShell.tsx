import { useEffect, useMemo, useState } from "react";

import { ImportPage } from "../pages/ImportPage";
import { ReaderPage } from "../pages/ReaderPage";
import { AiSettingsPage } from "../pages/AiSettingsPage";
import { SubscriptionsPage } from "../pages/SubscriptionsPage";
import { getAppHealth, isTauriRuntime, type HealthReport } from "../services/tauriApi";
import type { AppTab } from "../store";

const TABS: Array<{ id: AppTab; label: string }> = [
  { id: "reader", label: "文章" },
  { id: "subscriptions", label: "订阅" },
  { id: "import", label: "导入" },
  { id: "settings", label: "设置" },
];

export function AppShell() {
  const [activeTab, setActiveTab] = useState<AppTab>("reader");
  const [health, setHealth] = useState<HealthReport>({});
  const [error, setError] = useState("");

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }
    getAppHealth()
      .then((report) => setHealth(report))
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : "无法获取后端健康状态");
      });
  }, []);

  const page = useMemo(() => {
    if (activeTab === "reader") {
      return <ReaderPage />;
    }
    if (activeTab === "subscriptions") {
      return <SubscriptionsPage />;
    }
    if (activeTab === "import") {
      return <ImportPage />;
    }
    return <AiSettingsPage />;
  }, [activeTab]);

  return (
    <main className="workspace-shell" data-testid="app-shell">
      <aside className="workspace-nav">
        <div className="workspace-brand">
          <h1 className="app-title">RSSR</h1>
          <p className="tiny-muted">Folo 风格阅读工作台</p>
        </div>
        <nav className="tabs tabs-vertical" aria-label="导航标签">
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
        <div className="workspace-health">
          {Object.entries(health).map(([name, status]) => (
            <p key={name} className="tiny-muted">
              {name}: {status}
            </p>
          ))}
          {error ? <p className="inline-error">{error}</p> : null}
        </div>
      </aside>
      <section className="workspace-content">{page}</section>
    </main>
  );
}
