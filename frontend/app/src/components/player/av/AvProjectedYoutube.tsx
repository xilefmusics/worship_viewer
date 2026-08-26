import { useEffect, useRef, useState } from 'react'

import {
  AV_PLAYBACK_ERROR_AUTOPLAY,
  AV_PLAYBACK_ERROR_LOAD,
  AV_PLAYBACK_ERROR_PROVIDER_UNAVAILABLE,
  mediaActionsForCommand,
  playbackAckFromElement,
  safePlaybackError,
  shouldEmitPlaybackAck,
  youtubeProviderErrorCode,
} from '@/lib/player/av-projection-playback'
import type {
  AvProjectionAckError,
  AvProjectionCommand,
  AvProjectionPlaybackAck,
} from '@/lib/player/av-projection-protocol'
import { sanitizeYoutubeVideoId } from '@/lib/player/av-remote-url'
import {
  YOUTUBE_NOCOOKIE_HOST,
  YOUTUBE_PLAYER_BUFFERING,
  YOUTUBE_PLAYER_CUED,
  YOUTUBE_PLAYER_ENDED,
  YOUTUBE_PLAYER_PAUSED,
  YOUTUBE_PLAYER_PLAYING,
  loadYoutubeIframeApi,
  type YoutubeIframePlayer,
  type YoutubeIframePlayerCtor,
} from '@/lib/player/youtube-iframe-api'
import { cn } from '@/lib/utils'

import './player-av.css'

type AvProjectedYoutubeProps = {
  command: AvProjectionCommand
  onAck: (
    applied: boolean,
    playback?: AvProjectionPlaybackAck,
    error?: AvProjectionAckError,
  ) => void
  loadApi?: () => Promise<YoutubeIframePlayerCtor>
}

function youtubeSnapshot(
  player: YoutubeIframePlayer,
  status: AvProjectionPlaybackAck['status'],
  volume: number,
  muted: boolean,
  loop: boolean,
): AvProjectionPlaybackAck {
  const durationSec = player.getDuration()
  const durationMs =
    Number.isFinite(durationSec) && durationSec > 0 ? Math.round(durationSec * 1000) : null
  const currentSec = player.getCurrentTime()
  return playbackAckFromElement({
    status,
    currentTimeMs: Number.isFinite(currentSec) ? Math.round(currentSec * 1000) : 0,
    durationMs,
    seekableStartMs: 0,
    seekableEndMs: durationMs,
    volume,
    muted,
    loop,
  })
}

function statusFromPlayerState(state: number): AvProjectionPlaybackAck['status'] {
  if (state === YOUTUBE_PLAYER_PLAYING) return 'playing'
  if (state === YOUTUBE_PLAYER_PAUSED || state === YOUTUBE_PLAYER_CUED) return 'paused'
  if (state === YOUTUBE_PLAYER_BUFFERING) return 'loading'
  if (state === YOUTUBE_PLAYER_ENDED) return 'ended'
  return 'loading'
}

export function AvProjectedYoutube({
  command,
  onAck,
  loadApi = loadYoutubeIframeApi,
}: AvProjectedYoutubeProps) {
  const content = command.content.type === 'youtube' ? command.content : null
  const hostRef = useRef<HTMLDivElement | null>(null)
  const playerRef = useRef<YoutubeIframePlayer | null>(null)
  const lastAckRef = useRef<{ at: number; ack: AvProjectionPlaybackAck } | null>(null)
  const commandIdRef = useRef(command.commandId)
  const loopRef = useRef(command.playback?.loop === true)
  const onAckRef = useRef(onAck)
  const volumeRef = useRef(command.playback?.volume ?? 1)
  const mutedRef = useRef(command.playback?.muted === true)
  const playGenRef = useRef(0)
  const endedRef = useRef(false)
  const commandRef = useRef(command)
  const [endedAtCommandId, setEndedAtCommandId] = useState<number | null>(null)
  const hideVisual =
    command.screenState !== 'live' || endedAtCommandId === command.commandId

  useEffect(() => {
    onAckRef.current = onAck
  }, [onAck])

  useEffect(() => {
    commandRef.current = command
    commandIdRef.current = command.commandId
    loopRef.current = command.playback?.loop === true
    volumeRef.current = command.playback?.volume ?? 1
    mutedRef.current = command.playback?.muted === true
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
    const videoId = sanitizeYoutubeVideoId(content?.videoId)
    if (!content || !videoId) {
      playerRef.current?.destroy()
      playerRef.current = null
      if (content && !videoId) {
        emitAck(false, undefined, safePlaybackError(AV_PLAYBACK_ERROR_LOAD), true)
      }
      return
    }

    const host = hostRef.current
    if (!host) return
    let cancelled = false
    const gen = ++playGenRef.current

    void loadApi().then(
      (Player) => {
        if (cancelled || gen !== playGenRef.current) return
        playerRef.current?.destroy()
        const player = new Player(host, {
          videoId,
          host: YOUTUBE_NOCOOKIE_HOST,
          width: '100%',
          height: '100%',
          playerVars: {
            autoplay: 0,
            controls: 0,
            disablekb: 1,
            fs: 0,
            modestbranding: 1,
            rel: 0,
            playsinline: 1,
            origin: window.location.origin,
            enablejsapi: 1,
          },
          events: {
            onReady: (event) => {
              if (cancelled || gen !== playGenRef.current) return
              playerRef.current = event.target
              const actions = mediaActionsForCommand(commandRef.current)
              if (!actions) return
              event.target.setVolume(Math.round(actions.volume * 100))
              if (actions.muted) event.target.mute()
              else event.target.unMute()
              if (actions.seekMs != null) event.target.seekTo(actions.seekMs / 1000, true)
              if (actions.pause) event.target.pauseVideo()
              if (actions.play && !endedRef.current) {
                try {
                  event.target.playVideo()
                } catch {
                  emitAck(
                    false,
                    youtubeSnapshot(event.target, 'error', actions.volume, actions.muted, loopRef.current),
                    safePlaybackError(AV_PLAYBACK_ERROR_AUTOPLAY),
                    true,
                  )
                }
              } else {
                emitAck(
                  true,
                  youtubeSnapshot(event.target, 'paused', actions.volume, actions.muted, loopRef.current),
                  undefined,
                  true,
                )
              }
            },
            onStateChange: (event) => {
              if (cancelled || gen !== playGenRef.current) return
              if (event.data === YOUTUBE_PLAYER_ENDED) {
                if (loopRef.current) {
                  event.target.seekTo(0, true)
                  event.target.playVideo()
                  return
                }
                endedRef.current = true
                setEndedAtCommandId(commandIdRef.current)
                const snapshot = youtubeSnapshot(
                  event.target,
                  'ended',
                  volumeRef.current,
                  mutedRef.current,
                  loopRef.current,
                )
                event.target.destroy()
                playerRef.current = null
                emitAck(true, snapshot, undefined, true)
                return
              }
              endedRef.current = false
              emitAck(
                true,
                youtubeSnapshot(
                  event.target,
                  statusFromPlayerState(event.data),
                  volumeRef.current,
                  mutedRef.current,
                  loopRef.current,
                ),
                undefined,
                true,
              )
            },
            onError: (event) => {
              if (cancelled || gen !== playGenRef.current) return
              emitAck(
                false,
                youtubeSnapshot(
                  event.target,
                  'error',
                  volumeRef.current,
                  mutedRef.current,
                  loopRef.current,
                ),
                safePlaybackError(youtubeProviderErrorCode(event.data)),
                true,
              )
            },
          },
        })
        playerRef.current = player
      },
      () => {
        if (cancelled || gen !== playGenRef.current) return
        emitAck(false, undefined, safePlaybackError(AV_PLAYBACK_ERROR_PROVIDER_UNAVAILABLE), true)
      },
    )

    const clock = window.setInterval(() => {
      const player = playerRef.current
      if (!player || endedRef.current) return
      const state = player.getPlayerState()
      if (state !== YOUTUBE_PLAYER_PLAYING && state !== YOUTUBE_PLAYER_BUFFERING) return
      emitAck(
        true,
        youtubeSnapshot(
          player,
          statusFromPlayerState(state),
          volumeRef.current,
          mutedRef.current,
          loopRef.current,
        ),
        undefined,
        false,
      )
    }, 250)

    return () => {
      cancelled = true
      playGenRef.current += 1
      window.clearInterval(clock)
      playerRef.current?.destroy()
      playerRef.current = null
    }
    // Recreate only when the video identity changes; later effects apply play/pause.
    // eslint-disable-next-line react-hooks/exhaustive-deps -- command identity is videoId
  }, [content?.videoId, loadApi])

  useEffect(() => {
    const player = playerRef.current
    const actions = mediaActionsForCommand(command)
    if (!content) {
      player?.destroy()
      playerRef.current = null
      endedRef.current = false
      return
    }
    if (!actions || !player) return
    if (actions.release) {
      player.destroy()
      playerRef.current = null
      endedRef.current = false
      return
    }
    player.setVolume(Math.round(actions.volume * 100))
    if (actions.muted) player.mute()
    else player.unMute()
    if (actions.seekMs != null) player.seekTo(actions.seekMs / 1000, true)
    if (actions.pause) player.pauseVideo()
    if (actions.play && !endedRef.current) player.playVideo()
  }, [command, content])

  if (!content) return null

  return (
    <div
      className={cn('av-projected-media', hideVisual && 'av-projected-media--hidden')}
      data-testid="av-projected-youtube"
      aria-hidden={hideVisual || undefined}
    >
      <div ref={hostRef} className="av-projected-media__youtube" />
    </div>
  )
}
