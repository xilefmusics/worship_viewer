import { AvProjectedLivestream } from '@/components/player/av/AvProjectedLivestream'
import { AvProjectedMedia } from '@/components/player/av/AvProjectedMedia'
import { AvProjectedWebPage } from '@/components/player/av/AvProjectedWebPage'
import { AvProjectedYoutube } from '@/components/player/av/AvProjectedYoutube'
import { isTimedProjectionContent } from '@/lib/player/av-projection-protocol'
import type {
  AvProjectionAckError,
  AvProjectionCommand,
  AvProjectionPlaybackAck,
} from '@/lib/player/av-projection-protocol'

type AvProjectedRemoteLayerProps = {
  command: AvProjectionCommand
  onAck: (
    applied: boolean,
    playback?: AvProjectionPlaybackAck,
    error?: AvProjectionAckError,
  ) => void
}

export function AvProjectedRemoteLayer({ command, onAck }: AvProjectedRemoteLayerProps) {
  if (!isTimedProjectionContent(command.content)) return null
  switch (command.content.type) {
    case 'video':
    case 'audio':
      return <AvProjectedMedia command={command} onAck={onAck} />
    case 'youtube':
      return <AvProjectedYoutube command={command} onAck={onAck} />
    case 'livestream':
      return <AvProjectedLivestream command={command} onAck={onAck} />
    case 'web_page':
      return <AvProjectedWebPage command={command} onAck={onAck} />
  }
}
