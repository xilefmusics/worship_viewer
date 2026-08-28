import { useCallback, useEffect, useRef, useState } from 'react'

import {
  AV_PLAYBACK_ERROR_AUTOPLAY,
  AV_PLAYBACK_ERROR_LOAD,
  AV_PLAYBACK_ERROR_UNSUPPORTED,
  mediaActionsForCommand,
  mediaErrorCode,
  playbackAckFromElement,
  playbackError,
  releaseMediaElement,
  safePlaybackError,
  shouldEmitPlaybackAck,
} from '@/lib/player/av-projection-playback'
import type {
  AvProjectionAckError,
  AvProjectionCommand,
  AvProjectionPlaybackAck,
} from '@/lib/player/av-projection-protocol'
import { canPlayNativeHls, loadHlsModule, type HlsModule } from '@/lib/player/av-hls-client'
import { sanitizeLivestreamStreamType, sanitizeLivestreamUrl } from '@/lib/player/av-remote-url'
import { cn } from '@/lib/utils'

import './player-av.css'

type AvProjectedLivestreamProps = {
  command: AvProjectionCommand
  onAck: (
    applied: boolean,
    playback?: AvProjectionPlaybackAck,
    error?: AvProjectionAckError,
  ) => void
  importHls?: () => Promise<HlsModule>
  nativeHlsSupported?: (el: HTMLMediaElement) => boolean
}

function elementSnapshot(
  el: HTMLMediaElement,
  status: AvProjectionPlaybackAck['status'],
  loop: boolean,
): AvProjectionPlaybackAck {
  const durationMs =
    Number.isFinite(el.duration) && el.duration > 0 && el.duration !== Number.POSITIVE_INFINITY
      ? Math.round(el.duration * 1000)
      : null
  const seekable = el.seekable
  const seekableStartMs = seekable.length > 0 ? Math.round(seekable.start(0) * 1000) : 0
  const seekableEndMs =
    seekable.length > 0 ? Math.round(seekable.end(seekable.length - 1) * 1000) : durationMs
  return playbackAckFromElement({
    status,
    currentTimeMs: Math.round((el.currentTime || 0) * 1000),
    durationMs,
    seekableStartMs,
    seekableEndMs: seekableEndMs ?? seekableStartMs,
    volume: el.volume,
    muted: el.muted,
    loop,
  })
}

export function AvProjectedLivestream({
  command,
  onAck,
  importHls = loadHlsModule,
  nativeHlsSupported = canPlayNativeHls,
}: AvProjectedLivestreamProps) {
  const content = command.content.type === 'livestream' ? command.content : null
  const mediaRef = useRef<HTMLVideoElement | null>(null)
  const hlsRef = useRef<{ destroy: () => void } | null>(null)
  const lastAckRef = useRef<{ at: number; ack: AvProjectionPlaybackAck } | null>(null)
  const commandIdRef = useRef(command.commandId)
  const loopRef = useRef(command.playback?.loop === true)
  const playGenRef = useRef(0)
  const attachGenRef = useRef(0)
  const endedRef = useRef(false)
  const onAckRef = useRef(onAck)
  const commandRef = useRef(command)
  const [endedAtCommandId, setEndedAtCommandId] = useState<number | null>(null)
  const [hasVideoTrack, setHasVideoTrack] = useState(true)
  const hideVisual =
    !hasVideoTrack ||
    command.screenState !== 'live' ||
    endedAtCommandId === command.commandId

  useEffect(() => {
    onAckRef.current = onAck
  }, [onAck])

  useEffect(() => {
    commandRef.current = command
    commandIdRef.current = command.commandId
    loopRef.current = command.playback?.loop === true
  }, [command])

  const emitAck = (
    applied: boolean,
    playback: AvProjectionPlaybackAck | undefined,
    error: AvProjectionAckError | undefined,
    force: boolean,
  ) => {
    if (playback) {
      const now = Date.now()
      if (!shouldEmitPlaybackAck(lastAckRef.current, playback, now, force)) return
      lastAckRef.current = { at: now, ack: playback }
    }
    onAckRef.current(applied, playback, error)
  }

  useEffect(() => {
    const el = mediaRef.current
    if (!el) return

    const onTime = () => {
      if (endedRef.current) return
      emitAck(
        true,
        elementSnapshot(el, el.paused ? 'paused' : 'playing', loopRef.current),
        undefined,
        false,
      )
    }
    const onPlaying = () => {
      endedRef.current = false
      setEndedAtCommandId(null)
      emitAck(true, elementSnapshot(el, 'playing', loopRef.current), undefined, true)
    }
    const onPause = () => {
      if (endedRef.current) return
      emitAck(true, elementSnapshot(el, 'paused', loopRef.current), undefined, true)
    }
    const onWaiting = () => {
      emitAck(true, elementSnapshot(el, 'loading', loopRef.current), undefined, true)
    }
    const onEnded = () => {
      if (loopRef.current) {
        el.currentTime = 0
        void el.play()
        return
      }
      endedRef.current = true
      setEndedAtCommandId(commandIdRef.current)
      const snapshot = elementSnapshot(el, 'ended', loopRef.current)
      playGenRef.current += 1
      hlsRef.current?.destroy()
      hlsRef.current = null
      releaseMediaElement(el)
      emitAck(true, snapshot, undefined, true)
    }
    const onError = () => {
      emitAck(
        false,
        elementSnapshot(el, 'error', loopRef.current),
        playbackError(mediaErrorCode(el), 'The media could not be played.'),
        true,
      )
    }
    const onMeta = () => {
      setHasVideoTrack(el.videoWidth > 0)
    }

    el.addEventListener('timeupdate', onTime)
    el.addEventListener('durationchange', onTime)
    el.addEventListener('playing', onPlaying)
    el.addEventListener('pause', onPause)
    el.addEventListener('waiting', onWaiting)
    el.addEventListener('ended', onEnded)
    el.addEventListener('error', onError)
    el.addEventListener('loadedmetadata', onMeta)
    return () => {
      el.removeEventListener('timeupdate', onTime)
      el.removeEventListener('durationchange', onTime)
      el.removeEventListener('playing', onPlaying)
      el.removeEventListener('pause', onPause)
      el.removeEventListener('waiting', onWaiting)
      el.removeEventListener('ended', onEnded)
      el.removeEventListener('error', onError)
      el.removeEventListener('loadedmetadata', onMeta)
    }
  }, [content?.url])

  useEffect(() => {
    const el = mediaRef.current
    const url = sanitizeLivestreamUrl(content?.url)
    const streamType = sanitizeLivestreamStreamType(content?.streamType)
    if (!content || !el) {
      hlsRef.current?.destroy()
      hlsRef.current = null
      releaseMediaElement(el)
      endedRef.current = false
      return
    }
    if (!url || !streamType) {
      emitAck(false, undefined, safePlaybackError(AV_PLAYBACK_ERROR_LOAD), true)
      return
    }

    const gen = ++attachGenRef.current
    const useNative = streamType === 'direct' || nativeHlsSupported(el)

    if (useNative) {
      hlsRef.current?.destroy()
      hlsRef.current = null
      if (el.getAttribute('src') !== url) {
        endedRef.current = false
        lastAckRef.current = null
        el.src = url
      }
      return () => {
        attachGenRef.current += 1
      }
    }

    let cancelled = false
    void importHls().then(
      (mod) => {
        if (cancelled || gen !== attachGenRef.current) return
        const Hls = mod.default
        if (!Hls.isSupported()) {
          emitAck(false, undefined, safePlaybackError(AV_PLAYBACK_ERROR_UNSUPPORTED), true)
          return
        }
        hlsRef.current?.destroy()
        const hls = new Hls()
        hlsRef.current = hls
        hls.on(Hls.Events.ERROR, (_event: string, data: { fatal?: boolean }) => {
          if (cancelled || gen !== attachGenRef.current || !data?.fatal) return
          emitAck(
            false,
            elementSnapshot(el, 'error', loopRef.current),
            safePlaybackError(AV_PLAYBACK_ERROR_LOAD),
            true,
          )
        })
        endedRef.current = false
        lastAckRef.current = null
        hls.loadSource(url)
        hls.attachMedia(el)
        const actions = mediaActionsForCommand(commandRef.current)
        if (actions?.play && !endedRef.current) {
          void el.play().catch(() => undefined)
        }
      },
      () => {
        if (cancelled || gen !== attachGenRef.current) return
        emitAck(false, undefined, safePlaybackError(AV_PLAYBACK_ERROR_UNSUPPORTED), true)
      },
    )

    return () => {
      cancelled = true
      attachGenRef.current += 1
      hlsRef.current?.destroy()
      hlsRef.current = null
    }
  }, [content, importHls, nativeHlsSupported])

  useEffect(() => {
    const el = mediaRef.current
    if (!content) {
      releaseMediaElement(el)
      endedRef.current = false
      return
    }
    const actions = mediaActionsForCommand(command)
    if (!actions || !el) return
    if (actions.release) {
      playGenRef.current += 1
      hlsRef.current?.destroy()
      hlsRef.current = null
      releaseMediaElement(el)
      endedRef.current = false
      return
    }
    el.volume = actions.volume
    el.muted = actions.muted
    if (actions.seekMs != null) {
      const seekTo = actions.seekMs / 1000
      if (Number.isFinite(el.duration) && el.duration > 0 && el.duration !== Number.POSITIVE_INFINITY) {
        el.currentTime = seekTo
      }
    }
    if (actions.pause) {
      playGenRef.current += 1
      el.pause()
    }
    if (actions.play && !endedRef.current) {
      const gen = ++playGenRef.current
      void el.play().then(
        () => undefined,
        () => {
          if (gen !== playGenRef.current) return
          emitAck(
            false,
            elementSnapshot(el, 'error', loopRef.current),
            safePlaybackError(AV_PLAYBACK_ERROR_AUTOPLAY),
            true,
          )
        },
      )
    }
    return () => {
      playGenRef.current += 1
    }
  }, [command, content])

  useEffect(() => {
    const el = mediaRef.current
    return () => {
      attachGenRef.current += 1
      playGenRef.current += 1
      hlsRef.current?.destroy()
      hlsRef.current = null
      releaseMediaElement(el)
    }
  }, [])

  const setMediaRef = useCallback((el: HTMLVideoElement | null) => {
    mediaRef.current = el
  }, [])

  if (!content) return null

  return (
    <div
      className={cn('av-projected-media', hideVisual && 'av-projected-media--hidden')}
      data-testid="av-projected-livestream"
      data-stream-type={content.streamType}
      aria-hidden={hideVisual || undefined}
    >
      <video
        ref={setMediaRef}
        className="av-projected-media__element"
        playsInline
        preload="auto"
        data-testid="av-projected-livestream-video"
      />
    </div>
  )
}
