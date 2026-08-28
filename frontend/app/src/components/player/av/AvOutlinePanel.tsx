import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import type { AvOutlineRow } from '@/lib/player/av-lyric-slides'
import { cn } from '@/lib/utils'

import '../player-outline-list.css'
import './player-av.css'

type AvOutlinePanelProps = {
  rows: AvOutlineRow[]
  onSelectSlide: (slideIndex: number) => void
}

export function AvOutlinePanel({ rows, onSelectSlide }: AvOutlinePanelProps) {
  const { t } = useTranslation()
  const sections = useMemo(() => {
    const result: AvOutlineRow[][] = []
    for (const row of rows) {
      if (!row.isSubSlide || result.length === 0) result.push([])
      result[result.length - 1]!.push(row)
    }
    return result
  }, [rows])
  const [collapsedSections, setCollapsedSections] = useState<Set<number>>(
    () => new Set(sections.map((_, index) => index)),
  )

  const selectedSection = sections.findIndex((section) => section.some((row) => row.selected))

  if (rows.length === 0) {
    return null
  }

  return (
    <nav className="av-outline-panel" aria-label={t('player.av.outlineAria')}>
      <ul className="player-outline-list">
        {sections.map((section, sectionIndex) => {
          const [header, ...children] = section
          if (!header) return null
          const collapsed = collapsedSections.has(sectionIndex) && selectedSection !== sectionIndex
          return (
            <li key={`${header.slideIndex}-${header.label}`}>
              <button
                type="button"
                className={cn(
                  'player-outline-list__item',
                  header.selected && 'player-outline-list__item--selected',
                )}
                aria-current={header.selected ? 'true' : undefined}
                aria-expanded={children.length > 0 ? !collapsed : undefined}
                onClick={() => {
                  onSelectSlide(header.slideIndex)
                  if (children.length > 0) {
                    setCollapsedSections((current) => {
                      const next = new Set(current)
                      if (next.has(sectionIndex)) next.delete(sectionIndex)
                      else next.add(sectionIndex)
                      return next
                    })
                  }
                }}
              >
                {header.label}
              </button>
              {!collapsed
                ? children.map((row) => (
                    <button
                      key={`${row.slideIndex}-${row.label}`}
                      type="button"
                      className={cn(
                        'player-outline-list__item',
                        row.selected && 'player-outline-list__item--selected',
                        'player-outline-list__item--sub',
                      )}
                      aria-current={row.selected ? 'true' : undefined}
                      onClick={() => onSelectSlide(row.slideIndex)}
                    >
                      {row.label}
                    </button>
                  ))
                : null}
            </li>
          )
        })}
      </ul>
    </nav>
  )
}
