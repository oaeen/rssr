import { useEffect, useMemo, useState } from "react";

import { StatCard } from "../components/StatCard";
import { ImportPage } from "../pages/ImportPage";
import { ReaderPage } from "../pages/ReaderPage";
import { AiSettingsPage } from "../pages/AiSettingsPage";
import { SubscriptionsPage } from "../pages/SubscriptionsPage";
import { getAppHealth, isTauriRuntime, type HealthReport } from "../services/tauriApi";
import type { AppTab } from "../store";

const TABS: Array<{ id: AppTab; label: string }> = [
  { id: "subscriptions", label: "订阅" },
  { id: "import", label: "导入" },
  { id: "reader", label: "阅读" },
  { id: "ai", label: "AI 设置" },
];

export function AppShell() {
  const [activeTab, setActiveTab] = useState<AppTab>("subscriptions");
  const [health, setHealth] = useState<HealthReport>({});
  const [error, setError] = useState("");

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }
    let disposed = false;
    getAppHealth()
      .then((report) => {
        if (!disposed) {
          setHealth(report);
        }
      })
      .catch((err: unknown) => {
        if (!disposed) {
          setError(err instanceof Error ? err.message : "无法获取后端健康状态");
        }
      });
    return () => {
      disposed = true;
    };
  }, []);

  const page = useMemo(() => {
    if (activeTab === "subscriptions") {
      return <SubscriptionsPage />;
    }
    if (activeTab === "import") {
      return <ImportPage />;
    }
    if (activeTab === "reader") {
      return <ReaderPage />;
    }
    return <AiSettingsPage />;
  }, [activeTab]);

  return (
    <main className="app-shell" data-testid="app-shell">
      <section className="hero">
        <div className="hero-title-group">
          <h1 className="app-title">RSSR</h1>
          <p className="app-subtitle">跨平台 RSS + AI 翻译总结工作台</p>
        </div>
        <div className="hero-metrics">
          {Object.entries(health).map(([name, status]) => (
            <StatCard key={name} title={name} value={status} />
          ))}
          {error ? <p className="inline-error">{error}</p> : null}
        </div>
      </section>

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

      <section className="content-panel">{page}</section>
    </main>
  );
}
