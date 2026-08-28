import { describe, expect, it } from 'vitest'

import type { Media } from '@/api/media'
import {
  formatMediaDuration,
  isCreateMediaKind,
  isValidUrlMediaInput,
  mediaCanonicalUrl,
  mediaDisplayKind,
  sniffAssetUploadKind,
  urlContent,
} from '@/lib/media-display'

function media(content: Media['content']): Media {
  return { id: 'media:1', owner: 'team:1', title: 'Test', content }
}

describe('media display helpers', () => {
  it('maps content kinds and canonical identities', () => {
    expect(mediaDisplayKind(media({ type: 'youtube', video_id: 'abc', canonical_url: 'https://www.youtube.com/watch?v=abc' }))).toBe('youtube')
    expect(mediaDisplayKind(media({ type: 'spotify', resource_type: 'track', spotify_id: '4iV5W9uYEdYUVa79Axb7Rh', canonical_url: 'https://open.spotify.com/track/4iV5W9uYEdYUVa79Axb7Rh' }))).toBe('spotify')
    expect(mediaDisplayKind(media({ type: 'video', blob_id: 'b1', duration_ms: 1, width: 2, height: 3 }))).toBe('video')
    expect(mediaCanonicalUrl(media({ type: 'livestream', stream_type: 'hls', url: 'https://example.com/live.m3u8' }))).toBe('https://example.com/live.m3u8')
    expect(urlContent('youtube', 'https://youtu.be/dQw4w9WgXcQ')).toEqual({ type: 'youtube', url: 'https://youtu.be/dQw4w9WgXcQ' })
  })

  it('sniffs deck upload kinds from type and filename', () => {
    expect(sniffAssetUploadKind({ type: 'application/pdf', name: 'x.bin' })).toBe('pdf')
    expect(sniffAssetUploadKind({ type: '', name: 'chart.svg' })).toBe('svg')
    expect(sniffAssetUploadKind({ type: 'image/png', name: 'a.png' })).toBe('image')
    expect(sniffAssetUploadKind({ type: 'text/plain', name: 'notes.txt' })).toBeNull()
  })

  it('formats duration for preview metadata', () => {
    expect(formatMediaDuration(65000)).toBe('1:05')
    expect(formatMediaDuration(125000)).toBe('2:05')
  })

  it('falls back safely for unknown content', () => {
    expect(mediaDisplayKind(media({ type: 'future' } as never))).toBe('unknown')
  })

  it('validates HTTPS URL inputs without replacing server validation', () => {
    expect(isValidUrlMediaInput('youtube', 'https://youtu.be/abcdefghijk')).toBe(true)
    expect(isValidUrlMediaInput('youtube', 'https://youtube.example/watch?v=x')).toBe(false)
    expect(isValidUrlMediaInput('youtube', 'https://user:pass@youtu.be/abcdefghijk')).toBe(false)
    expect(isValidUrlMediaInput('spotify', 'https://open.spotify.com/track/4iV5W9uYEdYUVa79Axb7Rh?si=share')).toBe(true)
    expect(isValidUrlMediaInput('spotify', 'https://spotify.example/track/4iV5W9uYEdYUVa79Axb7Rh')).toBe(false)
  })

  it('does not expose legacy livestream and web-page records as creatable kinds', () => {
    expect(isCreateMediaKind('livestream')).toBe(false)
    expect(isCreateMediaKind('web_page')).toBe(false)
  })
})
