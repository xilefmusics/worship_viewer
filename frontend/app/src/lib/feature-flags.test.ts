import { afterEach, describe, expect, it, vi } from 'vitest'

import { isRoomsV2Enabled } from '@/lib/feature-flags'

describe('isRoomsV2Enabled', () => {
  afterEach(() => {
    vi.unstubAllEnvs()
    delete globalThis.__WORSHIP_RUNTIME__
  })

  it.each([
    [undefined, false],
    ['', false],
    ['false', false],
    ['1', false],
    ['TRUE', false],
    ['true', true],
  ] as const)('VITE_ROOMS_V2_ENABLED=%j → %s', (value, expected) => {
    if (value === undefined) {
      vi.stubEnv('VITE_ROOMS_V2_ENABLED', undefined as unknown as string)
    } else {
      vi.stubEnv('VITE_ROOMS_V2_ENABLED', value)
    }
    expect(isRoomsV2Enabled()).toBe(expected)
  })

  it('prefers runtime config over the Vite env', () => {
    vi.stubEnv('VITE_ROOMS_V2_ENABLED', 'true')
    globalThis.__WORSHIP_RUNTIME__ = { roomsV2Enabled: false }
    expect(isRoomsV2Enabled()).toBe(false)
    globalThis.__WORSHIP_RUNTIME__ = { roomsV2Enabled: true }
    expect(isRoomsV2Enabled()).toBe(true)
  })
})
