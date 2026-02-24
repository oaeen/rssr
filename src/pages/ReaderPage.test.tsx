import { render, screen } from "@testing-library/react";

import { formatEntryTime, ReaderPage, resolveDisplayTitles } from "./ReaderPage";

describe("ReaderPage", () => {
  it("renders fallback message outside tauri runtime", () => {
    render(<ReaderPage />);
    expect(screen.getByText("阅读面板")).toBeInTheDocument();
    expect(
      screen.getByText("当前是浏览器测试环境，Tauri 命令不可用。请在桌面端运行查看完整功能。"),
    ).toBeInTheDocument();
  });

  it("prefers translated title while preserving original title", () => {
    const translated = resolveDisplayTitles("<p>Original Title</p>", "中文标题");
    expect(translated.primary).toBe("中文标题");
    expect(translated.secondary).toBe("Original Title");

    const originalOnly = resolveDisplayTitles("Only Original", "");
    expect(originalOnly.primary).toBe("Only Original");
    expect(originalOnly.secondary).toBeNull();
  });

  it("formats entry time and handles invalid value", () => {
    expect(formatEntryTime("2026-02-24T00:00:00Z")).not.toBe("未知时间");
    expect(formatEntryTime("invalid-date")).toBe("未知时间");
    expect(formatEntryTime(undefined)).toBe("未知时间");
  });
});
