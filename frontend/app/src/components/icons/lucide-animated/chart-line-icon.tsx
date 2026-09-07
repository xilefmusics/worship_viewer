import type { Variants } from 'motion/react'
import { motion, useAnimation } from 'motion/react'
import type { HTMLAttributes } from 'react'
import { forwardRef, useCallback, useEffect, useImperativeHandle } from 'react'

import { cn } from '@/lib/utils'

/** Lucide `chart-line` paths + Motion — same pattern as Lucide Animated (https://github.com/pqoqubbw/icons). */
export interface ChartLineIconHandle {
  startAnimation: () => void
  stopAnimation: () => void
}

export interface ChartLineIconProps extends HTMLAttributes<HTMLDivElement> {
  size?: number
  isHovered?: boolean
}

const LINE_VARIANTS: Variants = {
  normal: {
    pathLength: 1,
    opacity: 1,
    transition: { duration: 0.2 },
  },
  animate: {
    pathLength: [0, 1],
    opacity: [0.4, 1],
    transition: { duration: 0.45, ease: 'easeInOut' },
  },
}

export const ChartLineIcon = forwardRef<ChartLineIconHandle, ChartLineIconProps>(
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
          <path d="M3 3v16a2 2 0 0 0 2 2h16" />
          <motion.path
            animate={controls}
            d="m7 16 4-5 4 3 5-7"
            variants={LINE_VARIANTS}
          />
        </svg>
      </div>
    )
  },
)

ChartLineIcon.displayName = 'ChartLineIcon'
