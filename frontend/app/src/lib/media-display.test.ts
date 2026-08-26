import { describe, expect, it } from 'vitest'

import type { Media } from '@/api/media'
import {
  isValidUrlMediaInput,
  mediaCanonicalUrl,
  mediaDisplayKind,
  mediaDisplayStatus,
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
