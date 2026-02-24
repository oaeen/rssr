export function isSettingsWindowMode(search = window.location.search): boolean {
  const params = new URLSearchParams(search);
  return params.get("window") === "settings" || params.get("settings") === "1";
}
