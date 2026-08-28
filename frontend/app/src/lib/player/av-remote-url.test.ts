import { describe, expect, it } from 'vitest'

import {
  livestreamStreamTypeFromUrl,
  sanitizeAvHttpsUrl,
  sanitizeLivestreamStreamType,
  sanitizeWebPageUrl,
  sanitizeYoutubeVideoId,
} from '@/lib/player/av-remote-url'

describe('av-remote-url', () => {
  it('accepts canonical 11-character YouTube ids only', () => {
    expect(sanitizeYoutubeVideoId('dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ')
    expect(sanitizeYoutubeVideoId('abcdefghijk')).toBe('abcdefghijk')
    expect(sanitizeYoutubeVideoId('short')).toBeNull()
    expect(sanitizeYoutubeVideoId('not valid!!')).toBeNull()
    expect(sanitizeYoutubeVideoId('javascript:alert(1)')).toBeNull()
  })

  it('accepts credential-free fragment-free HTTPS URLs', () => {
    expect(sanitizeAvHttpsUrl('https://example.com/live.m3u8')).toBe('https://example.com/live.m3u8')
    expect(sanitizeWebPageUrl('https://example.com/bulletin')).toBe('https://example.com/bulletin')
    expect(sanitizeAvHttpsUrl('http://example.com/x')).toBeNull()
    expect(sanitizeAvHttpsUrl('https://user:pass@example.com/x')).toBeNull()
    expect(sanitizeAvHttpsUrl('https://example.com/x#frag')).toBeNull()
    expect(sanitizeAvHttpsUrl('javascript:alert(1)')).toBeNull()
    expect(sanitizeAvHttpsUrl('data:text/html,hi')).toBeNull()
    expect(sanitizeAvHttpsUrl('blob:https://example.com/1')).toBeNull()
    expect(sanitizeAvHttpsUrl('file:///tmp/x')).toBeNull()
  })

  it('classifies HLS from the URL path and validates stream type tags', () => {
    expect(livestreamStreamTypeFromUrl('https://example.com/live.m3u8')).toBe('hls')
    expect(livestreamStreamTypeFromUrl('https://example.com/LIVE.M3U8')).toBe('hls')
    expect(livestreamStreamTypeFromUrl('https://example.com/live.mp4')).toBe('direct')
    expect(sanitizeLivestreamStreamType('hls')).toBe('hls')
    expect(sanitizeLivestreamStreamType('direct')).toBe('direct')
    expect(sanitizeLivestreamStreamType('rtmp')).toBeNull()
  })
})
