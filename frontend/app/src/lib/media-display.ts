import type { CreateMediaContent, Media } from '@/api/media'

export type UrlMediaKind = 'youtube' | 'livestream' | 'web_page'
export type MediaDisplayKind = UrlMediaKind | 'slide_deck' | 'video' | 'audio' | 'unknown'
export type MediaDisplayStatus = 'processing' | 'ready' | 'failed' | 'unknown'

const URL_KINDS = new Set<UrlMediaKind>(['youtube', 'livestream', 'web_page'])

export function isUrlMediaKind(value: string): value is UrlMediaKind {
  return URL_KINDS.has(value as UrlMediaKind)
}

export function mediaDisplayKind(media: Media): MediaDisplayKind {
  const type = (media.content as { type?: unknown } | null | undefined)?.type
  switch (type) {
    case 'youtube':
    case 'livestream':
    case 'web_page':
    case 'slide_deck':
    case 'video':
    case 'audio':
      return type
    default:
      return 'unknown'
  }
}

export function mediaDisplayStatus(media: Media): MediaDisplayStatus {
  switch ((media as { status?: unknown }).status) {
    case 'processing':
    case 'ready':
    case 'failed':
      return media.status
    default:
      return 'unknown'
  }
}

export function mediaCanonicalUrl(media: Media): string | null {
  const content = media.content
  if (!content) return null
  switch (content.type) {
    case 'youtube':
      return content.canonical_url
    case 'livestream':
    case 'web_page':
      return content.url
    default:
      return null
  }
}

export function urlContent(kind: UrlMediaKind, url: string): CreateMediaContent {
  return { type: kind, url }
}

export function isValidUrlMediaInput(kind: UrlMediaKind, rawUrl: string): boolean {
  try {
    const url = new URL(rawUrl.trim())
    if (url.protocol !== 'https:' || url.username || url.password) return false
    if (!url.hostname) return false
    if (kind !== 'youtube') return true
    const host = url.hostname.toLowerCase().replace(/^www\./, '')
    return host === 'youtube.com' || host === 'm.youtube.com' || host === 'youtu.be'
  } catch {
    return false
  }
}
