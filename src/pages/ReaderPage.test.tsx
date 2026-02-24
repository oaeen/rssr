import { render, screen } from "@testing-library/react";

import { buildSourceIconUrl, formatEntryTime, ReaderPage, resolveDisplayTitles } from "./ReaderPage";

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

  it("builds source icon url from site url or feed url", () => {
    expect(
      buildSourceIconUrl({
        id: 1,
        title: "Example",
        site_url: "https://example.com",
        feed_url: "https://example.com/feed.xml",
        category: null,
        is_active: true,
        failure_count: 0,
        created_at: "",
        updated_at: "",
      }),
    ).toContain("domain=example.com");

    expect(
      buildSourceIconUrl({
        id: 2,
        title: "Fallback",
        site_url: null,
        feed_url: "https://blog.example.org/rss",
        category: null,
        is_active: true,
        failure_count: 0,
        created_at: "",
        updated_at: "",
      }),
    ).toContain("domain=blog.example.org");
  });
});
