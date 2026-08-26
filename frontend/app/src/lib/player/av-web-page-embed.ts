export const WEB_PAGE_IFRAME_SANDBOX = 'allow-scripts'

export function webPageIframeSandboxTokens(value: string | null): string[] {
  return (value ?? '')
    .split(/\s+/)
    .map((token) => token.trim())
    .filter(Boolean)
}

export const WEB_PAGE_FORBIDDEN_SANDBOX = [
  'allow-same-origin',
  'allow-forms',
  'allow-popups',
  'allow-downloads',
  'allow-top-navigation',
  'allow-top-navigation-by-user-activation',
  'allow-modals',
  'allow-pointer-lock',
] as const
