import { toPlainText, toSafeHtml } from "./richText";

describe("rich text utils", () => {
  it("sanitizes html and strips scripts", () => {
    const html = toSafeHtml("<p>Hello</p><script>alert(1)</script>");
    expect(html).toContain("<p>Hello</p>");
    expect(html).not.toContain("<script>");
  });

  it("converts html to plain text", () => {
    const text = toPlainText("<p>Line <strong>One</strong></p>");
    expect(text).toBe("Line One");
  });
});
