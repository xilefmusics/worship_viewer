import {
  DndContext,
  KeyboardSensor,
  PointerSensor,
  TouchSensor,
  closestCenter,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core'
import { restrictToParentElement, restrictToVerticalAxis } from '@dnd-kit/modifiers'
import { SortableContext, arrayMove, sortableKeyboardCoordinates, useSortable, verticalListSortingStrategy } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { useRef } from 'react'
import { useTranslation } from 'react-i18next'

import { DeckPagePreview } from '@/components/media/DeckPagePreview'
import { Button } from '@/components/ui/button'

// Flow: M3 — pointer, touch, and keyboard reorder of draft pages

export type DeckEditorPage = { id: string; blob_id: string }

export function DeckPagesEditor({
  mediaId,
  pages,
  disabled,
  onReorder,
  onRemove,
  onReplace,
  onAdd,
}: {
  mediaId: string
  pages: DeckEditorPage[]
  disabled?: boolean
  onReorder: (pages: DeckEditorPage[]) => void
  onRemove: (id: string) => void
  onReplace: (id: string, file: File) => void
  onAdd: (files: File[]) => void
}) {
  const { t } = useTranslation()
  const addRef = useRef<HTMLInputElement>(null)
  const replaceRef = useRef<HTMLInputElement>(null)
  const replaceId = useRef<string | null>(null)
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(TouchSensor, { activationConstraint: { delay: 220, tolerance: 6 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  )

  function onDragEnd(event: DragEndEvent) {
    if (disabled) return
    const { active, over } = event
    if (!over || active.id === over.id) return
    const oldIndex = pages.findIndex((page) => page.id === active.id)
    const newIndex = pages.findIndex((page) => page.id === over.id)
    if (oldIndex < 0 || newIndex < 0) return
    onReorder(arrayMove(pages, oldIndex, newIndex))
  }

  return (
    <div className="grid gap-3">
      <div className="flex flex-wrap gap-2">
        <input
          ref={addRef}
          type="file"
          className="hidden"
          accept="image/png,image/jpeg,image/svg+xml,application/pdf,.png,.jpg,.jpeg,.svg,.pdf"
          multiple
          onChange={(event) => {
            const files = [...(event.target.files ?? [])]
            if (files.length) onAdd(files)
            event.target.value = ''
          }}
        />
        <input
          ref={replaceRef}
          type="file"
          className="hidden"
          accept="image/png,image/jpeg,image/svg+xml,application/pdf,.png,.jpg,.jpeg,.svg,.pdf"
          onChange={(event) => {
            const file = event.target.files?.[0]
            const pageId = replaceId.current
            if (file && pageId) onReplace(pageId, file)
            event.target.value = ''
            replaceId.current = null
          }}
        />
        <Button type="button" variant="outline" disabled={disabled} onClick={() => addRef.current?.click()}>
          {t('media.deck.addPages')}
        </Button>
      </div>
      {pages.length === 0 ? (
        <p className="text-sm text-[var(--color-muted-foreground)]">{t('media.deck.empty')}</p>
      ) : (
        <DndContext sensors={sensors} collisionDetection={closestCenter} modifiers={[restrictToVerticalAxis, restrictToParentElement]} onDragEnd={onDragEnd}>
          <SortableContext items={pages.map((page) => page.id)} strategy={verticalListSortingStrategy}>
            <ol className="grid gap-2" aria-label={t('media.deck.listAria')}>
              {pages.map((page, index) => (
                <SortableDeckPage
                  key={page.id}
                  mediaId={mediaId}
                  page={page}
                  index={index}
                  disabled={disabled}
                  onRemove={() => onRemove(page.id)}
                  onReplace={() => {
                    replaceId.current = page.id
                    replaceRef.current?.click()
                  }}
                />
              ))}
            </ol>
          </SortableContext>
        </DndContext>
      )}
    </div>
  )
}

function SortableDeckPage({
  mediaId,
  page,
  index,
  disabled,
  onRemove,
  onReplace,
}: {
  mediaId: string
  page: DeckEditorPage
  index: number
  disabled?: boolean
  onRemove: () => void
  onReplace: () => void
}) {
  const { t } = useTranslation()
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: page.id,
    disabled,
  })
  const label = t('media.deck.pageLabel', { number: index + 1 })
  return (
    <li
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      className={`grid grid-cols-[auto_1fr_auto] items-center gap-3 rounded-lg border border-[var(--color-border)] p-2 ${isDragging ? 'opacity-70' : ''}`}
    >
      <button type="button" className="cursor-grab px-1 text-lg text-[var(--color-muted-foreground)]" aria-label={t('media.deck.reorderHandle', { number: index + 1 })} disabled={disabled} {...attributes} {...listeners}>
        ::
      </button>
      <DeckPagePreview mediaId={mediaId} blobId={page.blob_id} label={label} />
      <div className="grid gap-1">
        <Button type="button" size="sm" variant="outline" disabled={disabled} onClick={onReplace}>{t('media.deck.replacePage')}</Button>
        <Button type="button" size="sm" variant="outline" disabled={disabled} onClick={onRemove}>{t('media.deck.removePage')}</Button>
      </div>
    </li>
  )
}
