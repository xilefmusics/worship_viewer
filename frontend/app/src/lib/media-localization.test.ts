import { describe, expect, it } from 'vitest'

import de from '@/i18n/de.json'
import en from '@/i18n/en.json'

function leafKeys(value: unknown, prefix = ''): string[] {
  if (!value || typeof value !== 'object') return [prefix]
  return Object.entries(value).flatMap(([key, child]) => leafKeys(child, prefix ? `${prefix}.${key}` : key))
}

describe('media localization', () => {
  it('has matching non-empty English and German media keys', () => {
    expect(leafKeys(de.media).sort()).toEqual(leafKeys(en.media).sort())
    for (const value of [en.media, de.media]) {
      const strings = Object.values(value).flatMap((entry) => typeof entry === 'string' ? [entry] : Object.values(entry).filter((item): item is string => typeof item === 'string'))
      expect(strings.every((text) => text.trim().length > 0)).toBe(true)
    }
  })
})
