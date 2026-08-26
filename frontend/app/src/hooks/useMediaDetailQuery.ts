import { useQuery, useQueryClient } from '@tanstack/react-query'

import { fetchMedia, mediaDetailKey } from '@/api/media'

export function useMediaDetailQuery(mediaId: string) {
  const queryClient = useQueryClient()
  return useQuery({
    queryKey: mediaDetailKey(mediaId),
    enabled: Boolean(mediaId),
    queryFn: ({ signal }) => fetchMedia(queryClient, mediaId, signal),
  })
}
