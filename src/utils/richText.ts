import DOMPurify from "dompurify";

export function toSafeHtml(input: string | null | undefined): string {
  if (!input) {
    return "";
  }

  const text = input.trim();
  if (!text) {
    return "";
  }

  if (looksLikeHtml(text)) {
    return DOMPurify.sanitize(text);
  }

  return DOMPurify.sanitize(
    text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/\n/g, "<br/>"),
  );
}

export function toPlainText(input: string | null | undefined): string {
  if (!input) {
    return "";
  }
  const container = document.createElement("div");
  container.innerHTML = toSafeHtml(input);
  return container.textContent?.trim() ?? "";
}

function looksLikeHtml(value: string): boolean {
  return /<\/?[a-z][\s\S]*>/i.test(value);
}
