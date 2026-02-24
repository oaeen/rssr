import { render, screen } from "@testing-library/react";

import { AiSettingsPage } from "./AiSettingsPage";

describe("AiSettingsPage", () => {
  it("renders fallback message outside tauri runtime", () => {
    render(<AiSettingsPage />);
    expect(screen.getByText("AI Provider 设置")).toBeInTheDocument();
    expect(
      screen.getByText("当前是浏览器测试环境，Tauri 命令不可用。请在桌面端运行查看完整功能。"),
    ).toBeInTheDocument();
  });
});
