import { render, screen } from "@testing-library/react";

import { SubscriptionsPage } from "./SubscriptionsPage";

describe("SubscriptionsPage", () => {
  it("renders fallback message outside tauri runtime", () => {
    render(<SubscriptionsPage />);
    expect(screen.getByText("订阅管理")).toBeInTheDocument();
    expect(
      screen.getByText("当前是浏览器测试环境，Tauri 命令不可用。请在桌面端运行查看完整功能。"),
    ).toBeInTheDocument();
  });
});
