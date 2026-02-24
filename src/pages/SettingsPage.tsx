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
      <nav className="settings-tabs" aria-label="设置导航">
        {SETTINGS_TABS.map((tab) => (
          <button
            key={tab.id}
            className={activeTab === tab.id ? "tab tab-active" : "tab"}
            onClick={() => setActiveTab(tab.id)}
            type="button"
          >
            {tab.label}
          </button>
        ))}
      </nav>
      <section className="settings-body">{content}</section>
    </section>
  );
}
