import { isSettingsWindowMode } from "./windowMode";

describe("window mode", () => {
  it("detects settings window by query string", () => {
    expect(isSettingsWindowMode("?window=settings")).toBe(true);
    expect(isSettingsWindowMode("?settings=1")).toBe(true);
    expect(isSettingsWindowMode("?window=main")).toBe(false);
  });
});
