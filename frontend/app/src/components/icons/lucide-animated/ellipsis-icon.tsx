import type { Variants } from 'motion/react'
import { motion, useAnimation } from 'motion/react'
import type { HTMLAttributes } from 'react'
import { forwardRef, useCallback, useEffect, useImperativeHandle } from 'react'

import { cn } from '@/lib/utils'

/** Lucide ellipsis paths + Motion — same pattern as Lucide Animated (https://github.com/pqoqubbw/icons). */
export interface EllipsisIconHandle {
  startAnimation: () => void
  stopAnimation: () => void
}

export interface EllipsisIconProps extends HTMLAttributes<HTMLDivElement> {
  size?: number
  isHovered?: boolean
}

const GROUP_VARIANTS: Variants = {
  normal: {},
  animate: {
    transition: { staggerChildren: 0.08 },
  },
}

const DOT_VARIANTS: Variants = {
  normal: { y: 0 },
  animate: {
    y: [0, -2.5, 0],
    transition: { duration: 0.4, ease: 'easeInOut' },
  },
}

export const EllipsisIcon = forwardRef<EllipsisIconHandle, EllipsisIconProps>(
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
          <motion.g animate={controls} initial="normal" variants={GROUP_VARIANTS}>
            <motion.circle cx="5" cy="12" r="1" variants={DOT_VARIANTS} />
            <motion.circle cx="12" cy="12" r="1" variants={DOT_VARIANTS} />
            <motion.circle cx="19" cy="12" r="1" variants={DOT_VARIANTS} />
          </motion.g>
        </svg>
      </div>
    )
  },
)

EllipsisIcon.displayName = 'EllipsisIcon'
