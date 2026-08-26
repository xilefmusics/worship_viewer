import { describe, expect, it } from 'vitest'

import type { Media } from '@/api/media'
import {
  formatMediaDuration,
  hasReplacementFailure,
  isProcessingActive,
  isReadyUploaded,
  isValidUrlMediaInput,
  mediaCanonicalUrl,
  mediaDisplayKind,
  mediaDisplayStatus,
  sniffAssetUploadKind,
  urlContent,
} from '@/lib/media-display'

function media(overrides: Partial<Media>): Media {
  return { id: 'media:1', owner: 'team:1', title: 'Test', status: 'ready', ...overrides }
}

describe('media display helpers', () => {
  it('maps all URL content and canonical identities', () => {
    expect(mediaDisplayKind(media({ content: { type: 'youtube', video_id: 'abc', canonical_url: 'https://www.youtube.com/watch?v=abc' } }))).toBe('youtube')
    expect(mediaCanonicalUrl(media({ content: { type: 'livestream', stream_type: 'hls', url: 'https://example.com/live.m3u8' } }))).toBe('https://example.com/live.m3u8')
    expect(urlContent('web_page', 'https://example.com')).toEqual({ type: 'web_page', url: 'https://example.com' })
  })

  it('uses declared_kind when content is absent', () => {
    expect(mediaDisplayKind(media({ status: 'processing', content: undefined, declared_kind: 'video' }))).toBe('video')
    expect(mediaDisplayKind(media({ status: 'processing', content: undefined, declared_kind: 'audio' }))).toBe('audio')
    expect(mediaDisplayKind(media({ status: 'processing', content: undefined, declared_kind: 'slide_deck' }))).toBe('slide_deck')
  })

  it('treats a draft-idle slide deck as not actively processing', () => {
    const idleDraft = media({
      status: 'processing',
      declared_kind: 'slide_deck',
      pending_revision: {
        operation: 'rev1',
        status: 'ready',
        pages: [{ id: 'p1', blob_id: 'b1' }],
      },
    })
    expect(isProcessingActive(idleDraft)).toBe(false)
    expect(isReadyUploaded(media({
      status: 'ready',
      content: { type: 'slide_deck', pages: [{ blob_id: 'b1' }] },
    }))).toBe(true)
  })

  it('sniffs deck upload kinds from type and filename', () => {
    expect(sniffAssetUploadKind({ type: 'application/pdf', name: 'x.bin' })).toBe('pdf')
    expect(sniffAssetUploadKind({ type: '', name: 'chart.svg' })).toBe('svg')
    expect(sniffAssetUploadKind({ type: 'image/png', name: 'a.png' })).toBe('image')
    expect(sniffAssetUploadKind({ type: 'text/plain', name: 'notes.txt' })).toBeNull()
  })

  it('detects processing and replacement lifecycle states', () => {
    const processing = media({ status: 'processing', declared_kind: 'video' })
    expect(isProcessingActive(processing)).toBe(true)
    expect(isReadyUploaded(processing)).toBe(false)

    const readyVideo = media({
      content: { type: 'video', blob_id: 'a1', duration_ms: 1000, width: 640, height: 360 },
    })
    expect(isReadyUploaded(readyVideo)).toBe(true)

    const replacementFailed = media({
      content: { type: 'video', blob_id: 'a1', duration_ms: 1000, width: 640, height: 360 },
      pending_revision: {
        operation: 'op1',
        status: 'failed',
        processing_error: { code: 'media_processing_failed', detail: 'Processing failed.' },
      },
    })
    expect(hasReplacementFailure(replacementFailed)).toBe(true)
    expect(isProcessingActive(replacementFailed)).toBe(false)
  })

  it('formats duration for preview metadata', () => {
    expect(formatMediaDuration(65000)).toBe('1:05')
    expect(formatMediaDuration(125000)).toBe('2:05')
  })

  it('falls back safely for unknown content and lifecycle values', () => {
    expect(mediaDisplayKind(media({ content: { type: 'future' } as never }))).toBe('unknown')
    expect(mediaDisplayStatus(media({ status: 'future' as never }))).toBe('unknown')
  })

  it('validates HTTPS URL inputs without replacing server validation', () => {
    expect(isValidUrlMediaInput('youtube', 'https://youtu.be/abcdefghijk')).toBe(true)
    expect(isValidUrlMediaInput('youtube', 'https://youtube.example/watch?v=x')).toBe(false)
    expect(isValidUrlMediaInput('livestream', 'http://example.com/live')).toBe(false)
    expect(isValidUrlMediaInput('web_page', 'https://user:pass@example.com')).toBe(false)
  })
})
