import type { CreateMediaContent, Media } from '@/api/media'

export type UrlMediaKind = 'youtube' | 'livestream' | 'web_page'
export type UploadMediaKind = 'video' | 'audio'
export type CreateMediaKind = UrlMediaKind | UploadMediaKind
export type MediaDisplayKind = CreateMediaKind | 'slide_deck' | 'unknown'
export type MediaDisplayStatus = 'processing' | 'ready' | 'failed' | 'unknown'

const URL_KINDS = new Set<UrlMediaKind>(['youtube', 'livestream', 'web_page'])
const UPLOAD_KINDS = new Set<UploadMediaKind>(['video', 'audio'])

export function isUrlMediaKind(value: string): value is UrlMediaKind {
  return URL_KINDS.has(value as UrlMediaKind)
}

export function isUploadMediaKind(value: string): value is UploadMediaKind {
  return UPLOAD_KINDS.has(value as UploadMediaKind)
}

export function isCreateMediaKind(value: string): value is CreateMediaKind {
  return isUrlMediaKind(value) || isUploadMediaKind(value)
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
      if (media.declared_kind === 'video') return 'video'
      if (media.declared_kind === 'audio') return 'audio'
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

export function isProcessingActive(media: Media): boolean {
  return (
    media.status === 'processing' ||
    media.pending_revision?.status === 'processing'
  )
}

export function isUploadedDisplayKind(kind: MediaDisplayKind): boolean {
  return kind === 'video' || kind === 'audio'
}

export function isReadyUploaded(media: Media): boolean {
  return media.status === 'ready' && isUploadedDisplayKind(mediaDisplayKind(media))
}

export function hasReplacementFailure(media: Media): boolean {
  return (
    media.status === 'ready' &&
    media.pending_revision?.status === 'failed'
  )
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

export function uploadCreateContent(kind: UploadMediaKind): CreateMediaContent {
  return { type: kind }
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

export function formatMediaDuration(durationMs: number): string {
  const totalSeconds = Math.max(0, Math.round(durationMs / 1000))
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60
  return `${minutes}:${seconds.toString().padStart(2, '0')}`
}
