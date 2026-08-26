import { useTranslation } from 'react-i18next'

import { AvBackgroundSelector } from '@/components/player/av/AvBackgroundSelector'
import { Button } from '@/components/ui/button'
import type {
  AvBackgroundLayer,
  AvBackgroundPreset,
  AvContentLayer,
} from '@/lib/player/av-preferences'
import { formatAvClock } from '@/lib/player/av-projection-playback'
import type { AvProjectionPlaybackAction } from '@/lib/player/av-projection-protocol'
import type { AvAggregatedPlayback } from '@/lib/player/av-projection-reducer'

import './player-av.css'

type AvMediaTransportPanelProps = {
  kind: 'video' | 'audio'
  title: string
  projected: boolean
  issuedAction?: AvProjectionPlaybackAction | null
  playback: AvAggregatedPlayback
  volume: number
  muted: boolean
  loop: boolean
  backgroundLayer: AvBackgroundLayer
  backgroundPreviewText: string
  contentLayer: AvContentLayer
  onPlay: () => void
  onPause: () => void
  onResume: () => void
  onSeek: (positionMs: number) => void
  onRestart: () => void
  onVolume: (volume: number) => void
  onMute: (muted: boolean) => void
  onLoop: (loop: boolean) => void
  onRetry: () => void
  onSelectBackgroundPreset: (preset: AvBackgroundPreset) => void
}

export function AvMediaTransportPanel({
  kind,
  title,
  projected,
  issuedAction = null,
  playback,
  volume,
  muted,
  loop,
  backgroundLayer,
  backgroundPreviewText,
  contentLayer,
  onPlay,
  onPause,
  onResume,
  onSeek,
  onRestart,
  onVolume,
  onMute,
  onLoop,
  onRetry,
  onSelectBackgroundPreset,
}: AvMediaTransportPanelProps) {
  const { t } = useTranslation()
  const playing = playback.status === 'playing'
  const failed = playback.status === 'error' || playback.errorCount > 0
  const duration = playback.durationMs ?? playback.seekableEndMs
  const canSeek = projected && duration > 0
  const optimisticPlaying =
    projected &&
    (issuedAction === 'play' || issuedAction === 'resume' || issuedAction === 'restart') &&
    playback.status === 'idle'
  const primaryAction =
    !projected || playback.status === 'ended'
      ? 'play'
      : playing || optimisticPlaying
        ? 'pause'
        : 'resume'

  const statusLabel = !projected
    ? t('player.av.mediaPreviewOnly')
    : playback.status === 'playing'
      ? t('player.av.mediaPlaying')
      : playback.status === 'paused'
        ? t('player.av.mediaPaused')
        : playback.status === 'ended'
          ? t('player.av.mediaEnded')
          : playback.status === 'loading'
            ? t('player.av.mediaLoading')
            : playback.status === 'error'
              ? t('player.av.mediaError')
              : playback.status === 'mixed'
                ? t('player.av.mediaMixed')
                : t('player.av.mediaPreviewOnly')

  const failedOutputs = playback.outputs.filter(
    (output) => output.outputStatus === 'failed' || output.playback?.status === 'error',
  )

  return (
    <div className="av-slides-panel-shell">
      <div className="av-media-transport" data-testid="av-media-transport" data-kind={kind}>
        <p className="av-media-transport__title">{title}</p>
        <p className="av-media-transport__kind">
          {kind === 'video' ? t('media.kinds.video') : t('media.kinds.audio')}
        </p>
        <p className="av-media-transport__status" data-testid="av-media-status">
          {statusLabel}
        </p>

        <div className="av-media-transport__controls">
          {primaryAction === 'play' ? (
            <Button type="button" onClick={onPlay}>
              {t('player.av.play')}
            </Button>
          ) : primaryAction === 'pause' ? (
            <Button type="button" onClick={onPause}>
              {t('player.av.pause')}
            </Button>
          ) : (
            <Button type="button" onClick={onResume}>
              {t('player.av.resume')}
            </Button>
          )}
          <Button type="button" variant="outline" onClick={onRestart} disabled={!projected}>
            {t('player.av.restart')}
          </Button>
          <Button
            type="button"
            variant={loop ? 'default' : 'outline'}
            aria-pressed={loop}
            onClick={() => onLoop(!loop)}
          >
            {t('player.av.loop')}
          </Button>
        </div>

        <label className="av-media-transport__slider">
          <span className="av-media-transport__slider-label">{t('player.av.seek')}</span>
          <input
            type="range"
            min={playback.seekableStartMs}
            max={Math.max(playback.seekableStartMs, duration)}
            step={100}
            value={Math.min(playback.currentTimeMs, duration || playback.currentTimeMs)}
            disabled={!canSeek}
            aria-label={t('player.av.seek')}
            onChange={(event) => onSeek(Number(event.target.value))}
          />
          <span className="av-media-transport__clock">
            {formatAvClock(playback.currentTimeMs)}
            {duration > 0 ? ` / ${formatAvClock(duration)}` : ''}
          </span>
        </label>

        <div className="av-media-transport__volume-row">
          <label className="av-media-transport__slider">
            <span className="av-media-transport__slider-label">{t('player.av.volume')}</span>
            <input
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={muted ? 0 : volume}
              aria-label={t('player.av.volume')}
              onChange={(event) => onVolume(Number(event.target.value))}
            />
          </label>
          <Button
            type="button"
            variant="outline"
            size="sm"
            aria-pressed={muted}
            onClick={() => onMute(!muted)}
          >
            {muted ? t('player.av.unmute') : t('player.av.mute')}
          </Button>
        </div>

        {failed ? (
          <div className="av-media-transport__error" role="alert">
            <p>{t('player.av.mediaAutoplayFailed')}</p>
            <Button type="button" variant="outline" size="sm" onClick={onRetry}>
              {t('player.av.retry')}
            </Button>
          </div>
        ) : null}

        {failedOutputs.length > 0 ? (
          <ul className="av-media-transport__outputs">
            {failedOutputs.map((output) => (
              <li key={output.outputId}>
                {t('player.av.outputFailed', {
                  id: output.outputId,
                  detail: output.error?.detail ?? output.playback?.status ?? '',
                })}
              </li>
            ))}
          </ul>
        ) : null}
      </div>
      <AvBackgroundSelector
        preset={backgroundLayer.preset}
        previewText={backgroundPreviewText}
        contentLayer={contentLayer}
        onSelectPreset={onSelectBackgroundPreset}
      />
    </div>
  )
}
