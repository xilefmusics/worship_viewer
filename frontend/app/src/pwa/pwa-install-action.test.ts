import { describe, expect, it } from 'vitest'

import { resolvePwaInstallAction } from '@/pwa/pwa-install-action'

const none = {
  isIos: false,
  isMacSafari: false,
  isAndroidChrome: false,
  isAndroidFirefox: false,
  hasNativePrompt: false,
}

describe('resolvePwaInstallAction', () => {
  it('prefers iOS help over every other signal', () => {
    expect(
      resolvePwaInstallAction({
        ...none,
        isIos: true,
        isAndroidChrome: true,
        hasNativePrompt: true,
      }),
    ).toEqual({ type: 'help', kind: 'ios' })
  })

  it('shows Mac Safari dock help when there is no native prompt', () => {
    expect(resolvePwaInstallAction({ ...none, isMacSafari: true })).toEqual({
      type: 'help',
      kind: 'safariMac',
    })
  })

  it('shows Firefox Android steps even if a native prompt exists', () => {
    expect(
      resolvePwaInstallAction({
        ...none,
        isAndroidFirefox: true,
        hasNativePrompt: true,
      }),
    ).toEqual({ type: 'help', kind: 'androidFirefox' })
  })

  it('uses the Chromium native prompt on Android Chrome when it is available', () => {
    expect(
      resolvePwaInstallAction({
        ...none,
        isAndroidChrome: true,
        hasNativePrompt: true,
      }),
    ).toEqual({ type: 'native-prompt' })
  })

  it('shows Chrome Android steps when the native prompt is missing', () => {
    expect(resolvePwaInstallAction({ ...none, isAndroidChrome: true })).toEqual({
      type: 'help',
      kind: 'androidChrome',
    })
  })

  it('falls back to generic help', () => {
    expect(resolvePwaInstallAction(none)).toEqual({ type: 'help', kind: 'generic' })
  })
})
