import { describe, expect, it } from 'vitest'

import { resolvePlayerMode } from '@/lib/player/player-mode'

describe('resolvePlayerMode', () => {
  it('uses explicit search mode when valid', () => {
    expect(resolvePlayerMode('av', 'sheet')).toBe('av')
    expect(resolvePlayerMode('sheet', 'av')).toBe('sheet')
  })

  it('falls back to global default when search mode is missing or invalid', () => {
    expect(resolvePlayerMode(undefined, 'av')).toBe('av')
    expect(resolvePlayerMode(null, 'sheet')).toBe('sheet')
    expect(resolvePlayerMode('invalid', 'av')).toBe('av')
  })

  it('does not accept removed legacy modes', () => {
    expect(resolvePlayerMode('legacy', 'av')).toBe('av')
  })
})
