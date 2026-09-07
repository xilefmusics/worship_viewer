import { motion, useAnimation } from 'motion/react'
import type { HTMLAttributes, ReactNode } from 'react'
import { useEffect } from 'react'

import { FileStackIcon } from '@/components/icons/lucide-animated/file-stack-icon'
import { DownloadIcon } from '@/components/icons/lucide-animated/download-icon'
import { InfoIcon } from '@/components/icons/lucide-animated/info-icon'
import { LogoutIcon } from '@/components/icons/lucide-animated/logout-icon'
import { SettingsIcon } from '@/components/icons/lucide-animated/settings-icon'
import { UsersIcon } from '@/components/icons/lucide-animated/users-icon'
import { cn } from '@/lib/utils'

/** Matches `size-4` (1rem) at default root font size. */
const PROFILE_MENU_ICON_PX = 16

const iconClass = 'size-4 shrink-0 text-[var(--color-muted-foreground)]'

const PROFILE_GLYPH_VARIANTS = {
  normal: { y: 0, rotate: 0 },
  animate: {
    y: -1,
    rotate: -2,
    transition: {
      type: 'spring' as const,
      stiffness: 260,
      damping: 14,
      mass: 0.8,
    },
  },
}

type ProfileMenuIconProps = {
  className?: string
  isHovered?: boolean
} & Omit<HTMLAttributes<HTMLDivElement>, 'children'>

export function IconSettings({ className, isHovered, ...rest }: ProfileMenuIconProps) {
  return (
    <SettingsIcon
      className={cn(iconClass, className)}
      isHovered={isHovered}
      size={PROFILE_MENU_ICON_PX}
      {...rest}
    />
  )
}

export function IconUsers({ className, isHovered, ...rest }: ProfileMenuIconProps) {
  return (
    <UsersIcon
      className={cn(iconClass, className)}
      isHovered={isHovered}
      size={PROFILE_MENU_ICON_PX}
      {...rest}
    />
  )
}

/** Install app — same glyph as Lucide `download`. */
export function IconInstall({ className, isHovered, ...rest }: ProfileMenuIconProps) {
  return (
    <DownloadIcon
      className={cn(iconClass, className)}
      isHovered={isHovered}
      size={PROFILE_MENU_ICON_PX}
      {...rest}
    />
  )
}

export function IconAbout({ className, isHovered, ...rest }: ProfileMenuIconProps) {
  return (
    <InfoIcon
      className={cn(iconClass, className)}
      isHovered={isHovered}
      size={PROFILE_MENU_ICON_PX}
      {...rest}
    />
  )
}

function AnimatedProfileGlyph({
  className,
  isHovered,
  children,
  ...rest
}: ProfileMenuIconProps & { children: ReactNode }) {
  const controls = useAnimation()

  useEffect(() => {
    void controls.start(isHovered ? 'animate' : 'normal')
  }, [controls, isHovered])

  return (
    <div className={cn(iconClass, className)} {...rest}>
      <svg
        aria-hidden
        fill="none"
        height={PROFILE_MENU_ICON_PX}
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="2"
        viewBox="0 0 24 24"
        width={PROFILE_MENU_ICON_PX}
        xmlns="http://www.w3.org/2000/svg"
      >
        <motion.g animate={controls} variants={PROFILE_GLYPH_VARIANTS}>
          {children}
        </motion.g>
      </svg>
    </div>
  )
}

export function IconTutorials({ className, isHovered, ...rest }: ProfileMenuIconProps) {
  return (
    <AnimatedProfileGlyph className={className} isHovered={isHovered} {...rest}>
      <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
      <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2Z" />
    </AnimatedProfileGlyph>
  )
}

export function IconAdminDashboard({ className, isHovered, ...rest }: ProfileMenuIconProps) {
  return (
    <AnimatedProfileGlyph className={className} isHovered={isHovered} {...rest}>
      <path d="M3 19h18" />
      <path d="M5 17V9" />
      <path d="M11 17V5" />
      <path d="M17 17v-6" />
      <path d="M5 9l6-4 6 4 4-2" />
    </AnimatedProfileGlyph>
  )
}

export function IconLogout({ className, isHovered, ...rest }: ProfileMenuIconProps) {
  return (
    <LogoutIcon
      className={cn(iconClass, className)}
      isHovered={isHovered}
      size={PROFILE_MENU_ICON_PX}
      {...rest}
    />
  )
}

export function IconMedia({ className, isHovered, ...rest }: ProfileMenuIconProps) {
  return (
    <FileStackIcon
      className={cn(iconClass, className)}
      isHovered={isHovered}
      size={PROFILE_MENU_ICON_PX}
      {...rest}
    />
  )
}
