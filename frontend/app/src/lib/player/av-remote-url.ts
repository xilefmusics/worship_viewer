const YOUTUBE_ID_PATTERN = /^[A-Za-z0-9_-]{11}$/
const BLOCKED_SCHEMES = new Set(['javascript:', 'data:', 'blob:', 'file:'])

export type AvLivestreamStreamType = 'hls' | 'direct'

function parsedHttpsUrl(raw: string): URL | null {
  const trimmed = raw.trim()
  if (!trimmed) return null
  try {
    const url = new URL(trimmed)
    if (BLOCKED_SCHEMES.has(url.protocol)) return null
    if (url.protocol !== 'https:') return null
    if (!url.hostname) return null
    if (url.username || url.password) return null
    if (url.hash) return null
    return url
  } catch {
    return null
  }
}

export function sanitizeYoutubeVideoId(value: unknown): string | null {
  if (typeof value !== 'string') return null
  const id = value.trim()
  return YOUTUBE_ID_PATTERN.test(id) ? id : null
}

export function sanitizeAvHttpsUrl(value: unknown): string | null {
  if (typeof value !== 'string') return null
  const url = parsedHttpsUrl(value)
  return url ? url.toString() : null
}

export function livestreamStreamTypeFromUrl(url: string): AvLivestreamStreamType {
  try {
    const path = new URL(url).pathname.toLowerCase()
    return path.endsWith('.m3u8') ? 'hls' : 'direct'
  } catch {
    return 'direct'
  }
}

export function sanitizeLivestreamUrl(value: unknown): string | null {
  return sanitizeAvHttpsUrl(value)
}

export function sanitizeWebPageUrl(value: unknown): string | null {
  return sanitizeAvHttpsUrl(value)
}

export function sanitizeLivestreamStreamType(value: unknown): AvLivestreamStreamType | null {
  return value === 'hls' || value === 'direct' ? value : null
}
