import { describe, expect, it } from 'vitest'

import { AV_OUTPUT_CSP, isAvOutputPath } from '@/lib/player/av-output-csp'

describe('av-output-csp', () => {
  it('matches only the AV output route', () => {
    expect(isAvOutputPath('/player/output')).toBe(true)
    expect(isAvOutputPath('/player/output?s=shared')).toBe(true)
    expect(isAvOutputPath('/player/output/')).toBe(true)
    expect(isAvOutputPath('/')).toBe(false)
    expect(isAvOutputPath('/player')).toBe(false)
    expect(isAvOutputPath('/api/v1/media')).toBe(false)
    expect(isAvOutputPath('/collections')).toBe(false)
  })

  it('allows YouTube, HTTPS frames, and media only on the output policy', () => {
    expect(AV_OUTPUT_CSP).toContain("default-src 'self'")
    expect(AV_OUTPUT_CSP).toContain('https://www.youtube.com')
    expect(AV_OUTPUT_CSP).toContain('https://www.youtube-nocookie.com')
    expect(AV_OUTPUT_CSP).toContain('frame-src https://www.youtube.com https://www.youtube-nocookie.com https:')
    expect(AV_OUTPUT_CSP).toContain("media-src 'self' blob: https:")
    expect(AV_OUTPUT_CSP).toContain("object-src 'none'")
    expect(AV_OUTPUT_CSP).toContain("form-action 'none'")
    expect(AV_OUTPUT_CSP).not.toContain("'unsafe-eval'")
    expect(AV_OUTPUT_CSP).not.toContain('allow-same-origin')
  })
})
