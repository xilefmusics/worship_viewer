import { parseProblemResponse } from '@/api/problem'

export type UploadMediaKind = 'video' | 'audio'

function apiBase(): string {
  return (import.meta.env.VITE_API_BASE_URL ?? '').replace(/\/$/, '')
}

function uploadUrl(mediaId: string, kind: UploadMediaKind): string {
  const base = apiBase()
  const path = `/api/v1/media/${encodeURIComponent(mediaId)}/uploads?kind=${kind}`
  return base ? `${base}${path}` : path
}

export function mediaAssetDataUrl(mediaId: string, assetId: string): string {
  const base = apiBase()
  const path = `/api/v1/media/${encodeURIComponent(mediaId)}/assets/${encodeURIComponent(assetId)}/data`
  return base ? `${base}${path}` : path
}

export type MediaUploadResult = { operation_id: string }

export async function uploadMediaSource(args: {
  mediaId: string
  kind: UploadMediaKind
  file: Blob
  onProgress?: (ratio: number) => void
  signal?: AbortSignal
}): Promise<MediaUploadResult> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest()
    xhr.open('PUT', uploadUrl(args.mediaId, args.kind))
    xhr.withCredentials = true
    xhr.responseType = 'text'
    xhr.setRequestHeader('Content-Type', args.file.type || 'application/octet-stream')
    if (args.signal) {
      if (args.signal.aborted) {
        reject(new DOMException('Aborted', 'AbortError'))
        return
      }
      args.signal.addEventListener('abort', () => xhr.abort())
    }
    xhr.upload.onprogress = (event) => {
      if (event.lengthComputable && args.onProgress) {
        args.onProgress(event.loaded / event.total)
      }
    }
    xhr.onload = async () => {
      if (xhr.status === 200) {
        try {
          resolve(JSON.parse(xhr.responseText) as MediaUploadResult)
        } catch {
          reject(new Error('upload_failed'))
        }
        return
      }
      if (xhr.status === 413) {
        reject(new Error('payload_too_large'))
        return
      }
      try {
        const problem = await parseProblemResponse(
          new Response(xhr.responseText, {
            status: xhr.status,
            headers: { 'Content-Type': 'application/problem+json' },
          }),
        )
        reject(new Error(problem?.detail ?? problem?.title ?? 'upload_failed'))
      } catch {
        reject(new Error('upload_failed'))
      }
    }
    xhr.onerror = () => reject(new Error('upload_failed'))
    xhr.onabort = () => reject(new DOMException('Aborted', 'AbortError'))
    xhr.send(args.file)
  })
}
