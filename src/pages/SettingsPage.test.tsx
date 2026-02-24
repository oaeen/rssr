import { fireEvent, render, screen } from "@testing-library/react";

import { SettingsPage } from "./SettingsPage";

describe("SettingsPage", () => {
  it("renders settings center and switches tabs", () => {
    render(<SettingsPage />);
    expect(screen.getByText("设置中心")).toBeInTheDocument();
    expect(screen.getByText("设置")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "订阅与导入" }));
    expect(screen.getByText("订阅管理")).toBeInTheDocument();
  });
});
