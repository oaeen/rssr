import { render, screen } from "@testing-library/react";

import App from "./App";

describe("App smoke", () => {
  it("renders app shell and title", () => {
    render(<App />);
    expect(screen.getByTestId("app-shell")).toBeInTheDocument();
    expect(screen.getByText("RSSR")).toBeInTheDocument();
  });
});
