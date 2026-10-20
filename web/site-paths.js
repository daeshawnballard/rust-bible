// Resolve public assets beside this module so the same Rust/WASM build works
// at the ChatGPT Site root and under a custom-domain project directory.
export const SITE_BASE_URL = new URL('./', import.meta.url);

export function mountHtml(html, basePath = SITE_BASE_URL.pathname) {
  if (basePath === '/') return html;
  return html
    .replace(/\b(src|poster|href)=(['"])\/(?!\/)/g,
      (_match, attribute, quote) => `${attribute}=${quote}${basePath}`)
    .replace(/url\((['"]?)\/(?!\/)/g,
      (_match, quote) => `url(${quote}${basePath}`);
}

export function mountApiHtml(value, basePath = SITE_BASE_URL.pathname) {
  if (Array.isArray(value)) return value.map(item => mountApiHtml(item, basePath));
  if (!value || typeof value !== 'object') return value;
  return Object.fromEntries(Object.entries(value).map(([key, item]) => [key,
    key === 'html' && typeof item === 'string' ? mountHtml(item, basePath)
      : mountApiHtml(item, basePath),
  ]));
}
