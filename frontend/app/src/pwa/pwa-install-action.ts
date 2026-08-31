export type PwaInstallHelpKind =
  | 'ios'
  | 'safariMac'
  | 'androidChrome'
  | 'androidFirefox'
  | 'generic'

export type PwaInstallAction =
  | { type: 'native-prompt' }
  | { type: 'help'; kind: PwaInstallHelpKind }

/** Choose native Chromium install vs an in-app help sheet. */
export function resolvePwaInstallAction(input: {
  isIos: boolean
  isMacSafari: boolean
  isAndroidChrome: boolean
  isAndroidFirefox: boolean
  hasNativePrompt: boolean
}): PwaInstallAction {
  if (input.isIos) {
    return { type: 'help', kind: 'ios' }
  }
  if (input.isMacSafari) {
    return { type: 'help', kind: 'safariMac' }
  }
  if (input.isAndroidFirefox) {
    return { type: 'help', kind: 'androidFirefox' }
  }
  if (input.hasNativePrompt) {
    return { type: 'native-prompt' }
  }
  if (input.isAndroidChrome) {
    return { type: 'help', kind: 'androidChrome' }
  }
  return { type: 'help', kind: 'generic' }
}
