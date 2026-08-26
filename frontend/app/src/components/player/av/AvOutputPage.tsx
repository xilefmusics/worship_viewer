import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { AvSlideView } from '@/components/player/av/AvSlideView'
import { DEFAULT_AV_PREFERENCES } from '@/lib/player/av-preferences'
import {
  slideViewPropsFromCommand,
  type AvProjectionCommand,
} from '@/lib/player/av-projection-protocol'
import {
  AV_OUTPUT_HEARTBEAT_MS,
  applyOutputCommand,
  outputAckForApply,
  type AvOutputProjectionState,
} from '@/lib/player/av-projection-reducer'
import {
  createAvProjectionChannel,
  readOrCreateAvOutputId,
} from '@/lib/player/av-projection-sync'

import './player-av.css'

type AvOutputPageProps = {
  sessionId: string
  allowFullscreenOnDblClick?: boolean
}

export function AvOutputPage({
  sessionId,
  allowFullscreenOnDblClick = true,
}: AvOutputPageProps) {
  const { t } = useTranslation()
  const [outputId] = useState(() => readOrCreateAvOutputId())
  const [state, setState] = useState<AvOutputProjectionState>({
    outputId,
    appliedCommandId: null,
    command: null,
  })
  const stateRef = useRef(state)
  const sendRef = useRef<((commandAck: ReturnType<typeof outputAckForApply>) => void) | null>(null)

  useEffect(() => {
    stateRef.current = state
  }, [state])

  useEffect(() => {
    const channel = createAvProjectionChannel(sessionId, (message) => {
      if (message.type !== 'command') return
      const result = applyOutputCommand(stateRef.current, message)
      if (!result.applied) return
      stateRef.current = result.state
      setState(result.state)
      if (result.state.command?.content.type === 'deck_page' && result.state.command.screenState === 'live') {
        return
      }
      const ack = outputAckForApply(result.state, true)
      if (ack) channel.send(ack)
    })
    sendRef.current = (ack) => {
      if (ack) channel.send(ack)
    }
    channel.send({ type: 'hello', sessionId, outputId, ready: true })
    const latest = channel.readLatestCommand()
    if (latest) {
      const result = applyOutputCommand(stateRef.current, latest)
      if (result.applied) {
        stateRef.current = result.state
        queueMicrotask(() => setState(result.state))
        if (!(result.state.command?.content.type === 'deck_page' && result.state.command.screenState === 'live')) {
          const ack = outputAckForApply(result.state, true)
          if (ack) channel.send(ack)
        }
      }
    }
    const heartbeat = window.setInterval(() => {
      channel.send({ type: 'heartbeat', sessionId, outputId, ready: true })
    }, AV_OUTPUT_HEARTBEAT_MS)
    return () => {
      window.clearInterval(heartbeat)
      channel.send({ type: 'goodbye', sessionId, outputId, ready: false })
      channel.close()
      sendRef.current = null
    }
  }, [outputId, sessionId])

  const command: AvProjectionCommand | null = state.command
  const view = command
    ? slideViewPropsFromCommand(command)
    : { contentText: '' }
  const screenState = command?.screenState ?? 'live'

  function onDoubleClick() {
    if (!allowFullscreenOnDblClick) return
    void document.documentElement.requestFullscreen?.()
  }

  return (
    <div
      className="h-dvh w-dvw overflow-hidden bg-black"
      onDoubleClick={onDoubleClick}
      aria-label={t('player.av.outputAria')}
    >
      <AvSlideView
        contentText={view.contentText}
        contentLines={view.contentLines}
        deckPage={view.deckPage}
        onDeckPageStatus={(status) => {
          if (stateRef.current.command?.content.type !== 'deck_page') return
          if (stateRef.current.command.screenState !== 'live') return
          sendRef.current?.(
            outputAckForApply(
              stateRef.current,
              status === 'ready',
              status === 'error'
                ? { code: 'asset_failed', detail: 'Deck page could not be loaded.' }
                : undefined,
            ),
          )
        }}
        contentLayer={command?.contentLayer ?? DEFAULT_AV_PREFERENCES.contentLayer}
        backgroundLayer={command?.backgroundLayer ?? DEFAULT_AV_PREFERENCES.backgroundLayer}
        transition={command?.transition ?? DEFAULT_AV_PREFERENCES.transition}
        screenState={screenState}
      />
    </div>
  )
}
