import { useEffect, useRef, useState } from 'react'

import {
  AV_PLAYBACK_ERROR_EMBED,
  AV_PLAYBACK_ERROR_LOAD,
  mediaActionsForCommand,
  playbackAckFromElement,
  safePlaybackError,
} from '@/lib/player/av-projection-playback'
import type {
  AvProjectionAckError,
  AvProjectionCommand,
  AvProjectionPlaybackAck,
} from '@/lib/player/av-projection-protocol'
import { sanitizeWebPageUrl } from '@/lib/player/av-remote-url'
import { WEB_PAGE_IFRAME_SANDBOX } from '@/lib/player/av-web-page-embed'
import { cn } from '@/lib/utils'

import './player-av.css'

type AvProjectedWebPageProps = {
  command: AvProjectionCommand
  onAck: (
    applied: boolean,
    playback?: AvProjectionPlaybackAck,
    error?: AvProjectionAckError,
  ) => void
}

function webSnapshot(status: AvProjectionPlaybackAck['status']): AvProjectionPlaybackAck {
  return playbackAckFromElement({
    status,
    currentTimeMs: 0,
    durationMs: null,
    seekableStartMs: 0,
    seekableEndMs: 0,
    volume: 0,
    muted: true,
    loop: false,
  })
}

export function AvProjectedWebPage({ command, onAck }: AvProjectedWebPageProps) {
  const content = command.content.type === 'web_page' ? command.content : null
  const onAckRef = useRef(onAck)
  const [loadKey, setLoadKey] = useState(0)
  const lastRestartAt = useRef<number | null>(null)
  const iframeRef = useRef<HTMLIFrameElement | null>(null)

  useEffect(() => {
    onAckRef.current = onAck
  }, [onAck])

  const url = sanitizeWebPageUrl(content?.url)
  const actions = content ? mediaActionsForCommand(command) : null
  const visible = Boolean(content && url && actions && actions.play && !actions.hide && !actions.release)

  useEffect(() => {
    if (!content) return
    if (!url) {
      onAckRef.current(false, webSnapshot('error'), safePlaybackError(AV_PLAYBACK_ERROR_LOAD))
      return
    }
    if (command.playback?.action === 'restart' && lastRestartAt.current !== command.commandId) {
      lastRestartAt.current = command.commandId
      setLoadKey((key) => key + 1)
    }
  }, [command.commandId, command.playback?.action, content, url])

  useEffect(() => {
    if (!content || !url) return
    if (actions?.release || actions?.hide || !actions?.play) {
      onAckRef.current(true, webSnapshot('paused'))
      return
    }
    onAckRef.current(true, webSnapshot('playing'))
  }, [actions?.hide, actions?.play, actions?.release, content, loadKey, url])

  useEffect(() => {
    const iframe = iframeRef.current
    if (!iframe) return
    const onError = () => {
      onAckRef.current(false, webSnapshot('error'), safePlaybackError(AV_PLAYBACK_ERROR_EMBED))
    }
    iframe.addEventListener('error', onError)
    return () => iframe.removeEventListener('error', onError)
  }, [visible, loadKey, url])

  if (!content) return null

  return (
    <div
      className={cn('av-projected-media', !visible && 'av-projected-media--hidden')}
      data-testid="av-projected-web"
      aria-hidden={!visible || undefined}
    >
      {visible ? (
        <iframe
          key={`${url}:${loadKey}`}
          ref={iframeRef}
          className="av-projected-media__iframe"
          title={command.itemTitle}
          src={url ?? undefined}
          sandbox={WEB_PAGE_IFRAME_SANDBOX}
          referrerPolicy="no-referrer"
          data-testid="av-projected-web-iframe"
        />
      ) : null}
    </div>
  )
}
