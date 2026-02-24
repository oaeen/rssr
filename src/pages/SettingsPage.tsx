import { useMemo, useState } from "react";

import { AiSettingsPage } from "./AiSettingsPage";
import { SubscriptionsPage } from "./SubscriptionsPage";

type SettingsTab = "system" | "subscriptions";

const SETTINGS_TABS: Array<{ id: SettingsTab; label: string }> = [
  { id: "system", label: "AI 与同步" },
  { id: "subscriptions", label: "订阅与导入" },
];

type SettingsPageProps = {
  onClose?: () => void;
};

export function SettingsPage({ onClose }: SettingsPageProps) {
  const [activeTab, setActiveTab] = useState<SettingsTab>("system");

  const content = useMemo(() => {
    if (activeTab === "system") {
      return <AiSettingsPage />;
    }
    return <SubscriptionsPage />;
  }, [activeTab]);

  return (
    <section className="settings-shell">
      <header className="settings-header">
        <div>
          <h2>设置中心</h2>
          <p className="tiny-muted">统一管理同步、AI、订阅和导入。</p>
        </div>
        {onClose ? (
          <button type="button" onClick={onClose}>
            返回阅读
          </button>
        ) : null}
      </header>
      <section className="settings-layout">
        <nav className="settings-nav" aria-label="设置导航">
          {SETTINGS_TABS.map((tab) => (
            <button
              key={tab.id}
              className={
                activeTab === tab.id
                  ? "settings-nav-item settings-nav-item-active"
                  : "settings-nav-item"
              }
              onClick={() => setActiveTab(tab.id)}
              type="button"
            >
              <span>{tab.label}</span>
              <small>{tab.id === "system" ? "模型、同步与状态" : "管理订阅和批量导入"}</small>
            </button>
          ))}
        </nav>
        <section className="settings-body">{content}</section>
      </section>
    </section>
  );
}
