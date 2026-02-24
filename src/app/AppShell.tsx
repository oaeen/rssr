import { useMemo, useState } from "react";

import { ReaderPage } from "../pages/ReaderPage";
import { SettingsPage } from "../pages/SettingsPage";

type ShellView = "reader" | "settings";

export function AppShell() {
  const [activeView, setActiveView] = useState<ShellView>("reader");

  const page = useMemo(() => {
    if (activeView === "reader") {
      return <ReaderPage onOpenSettings={() => setActiveView("settings")} />;
    }
    return <SettingsPage onClose={() => setActiveView("reader")} />;
  }, [activeView]);

  return (
    <main className="workspace-shell" data-testid="app-shell">
      <section className="workspace-content">{page}</section>
    </main>
  );
}
