/** Content-Security-Policy for `/player/output` HTML only. Keep in sync with backend `frontend.rs`. */
export const AV_OUTPUT_CSP = [
  "default-src 'self'",
  "script-src 'self' 'wasm-unsafe-eval' https://www.youtube.com https://www.youtube-nocookie.com https://s.ytimg.com",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: blob: https:",
  "font-src 'self'",
  "media-src 'self' blob: https:",
  "frame-src https://www.youtube.com https://www.youtube-nocookie.com https:",
  "connect-src 'self' https: blob: ws: wss:",
  "worker-src 'self' blob:",
  "object-src 'none'",
  "base-uri 'self'",
  "form-action 'none'",
  "frame-ancestors 'none'",
].join('; ')

/**
 * Vite injects the React Refresh preamble as an inline module in development.
 * Keep this allowance out of preview and production responses.
 */
export const AV_OUTPUT_DEV_CSP = AV_OUTPUT_CSP.replace(
  "script-src 'self'",
  "script-src 'self' 'unsafe-inline'",
)

export function isAvOutputPath(pathname: string): boolean {
  const path = pathname.split('?')[0] ?? ''
  return path === '/player/output' || path.startsWith('/player/output/')
}
