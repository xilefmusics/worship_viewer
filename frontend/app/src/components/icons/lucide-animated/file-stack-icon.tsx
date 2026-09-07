import type { Transition } from 'motion/react'
import { motion, useAnimation } from 'motion/react'
import type { HTMLAttributes } from 'react'
import { forwardRef, useCallback, useEffect, useImperativeHandle } from 'react'

import { cn } from '@/lib/utils'

/** Lucide Animated (MIT) — https://github.com/pqoqubbw/icons */
export interface FileStackIconHandle {
  startAnimation: () => void
  stopAnimation: () => void
}

export interface FileStackIconProps extends HTMLAttributes<HTMLDivElement> {
  size?: number
  isHovered?: boolean
}

const SPRING: Transition = {
  type: 'spring',
  stiffness: 160,
  damping: 17,
  mass: 1,
}

export const FileStackIcon = forwardRef<FileStackIconHandle, FileStackIconProps>(
  ({ onMouseEnter, onMouseLeave, className, size = 28, isHovered, ...props }, ref) => {
    const controls = useAnimation()
    const external = isHovered !== undefined

    useImperativeHandle(ref, () => ({
      startAnimation: () => controls.start('animate'),
      stopAnimation: () => controls.start('normal'),
    }))

    useEffect(() => {
      if (!external) return
      if (isHovered) {
        void controls.start('animate')
      } else {
        void controls.start('normal')
      }
    }, [external, isHovered, controls])

    const handleMouseEnter = useCallback(
      (e: React.MouseEvent<HTMLDivElement>) => {
        if (external) return
        void controls.start('animate')
        onMouseEnter?.(e)
      },
      [controls, external, onMouseEnter],
    )

    const handleMouseLeave = useCallback(
      (e: React.MouseEvent<HTMLDivElement>) => {
        if (external) return
        void controls.start('normal')
        onMouseLeave?.(e)
      },
      [controls, external, onMouseLeave],
    )

    return (
      <div
        className={cn(className)}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        {...props}
      >
        <svg
          fill="none"
          height={size}
          stroke="currentColor"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
          viewBox="0 0 24 24"
          width={size}
          xmlns="http://www.w3.org/2000/svg"
          aria-hidden
        >
          <motion.path
            animate={controls}
            d="M21 7h-3a2 2 0 0 1-2-2V2"
            transition={SPRING}
            variants={{
              normal: { translateX: 0, translateY: 0 },
              animate: { translateX: -4, translateY: 4 },
            }}
          />
          <motion.path
            animate={controls}
            d="M21 6v6.5c0 .8-.7 1.5-1.5 1.5h-7c-.8 0-1.5-.7-1.5-1.5v-9c0-.8.7-1.5 1.5-1.5H17Z"
            transition={SPRING}
            variants={{
              normal: { translateX: 0, translateY: 0 },
              animate: { translateX: -4, translateY: 4 },
            }}
          />
          <path d="M7 8v8.8c0 .3.2.6.4.8.2.2.5.4.8.4H15" />
          <motion.path
            animate={controls}
            d="M3 12v8.8c0 .3.2.6.4.8.2.2.5.4.8.4H11"
            transition={SPRING}
            variants={{
              normal: { translateX: 0, translateY: 0 },
              animate: { translateX: 4, translateY: -4 },
            }}
          />
        </svg>
      </div>
    )
  },
)

FileStackIcon.displayName = 'FileStackIcon'
