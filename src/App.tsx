import { AppShell } from "./app/AppShell";
import { isSettingsWindowMode } from "./app/windowMode";
import { AiSettingsPage } from "./pages/AiSettingsPage";
import "./App.css";

function App() {
  if (isSettingsWindowMode()) {
    return <AiSettingsPage />;
  }
  return <AppShell />;
}

export default App;
