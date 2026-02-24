import { render, screen } from "@testing-library/react";

import { ReaderPage } from "./ReaderPage";

describe("ReaderPage", () => {
  it("renders fallback message outside tauri runtime", () => {
    render(<ReaderPage />);
    expect(screen.getByText("阅读面板")).toBeInTheDocument();
    expect(
      screen.getByText("当前是浏览器测试环境，Tauri 命令不可用。请在桌面端运行查看完整功能。"),
    ).toBeInTheDocument();
  });
});
