import { Message } from '@/util/Message'

export async function getAudioMediaStream (): Promise<MediaStream> {
  const mediaStream = await window.navigator.mediaDevices
    .getUserMedia({
      audio: true,
      video: false
    })
  return mediaStream
}

// eslint-disable-next-line
export function createGetAudioMediaStreamErrMessage (e: any): string {
  if (e instanceof DOMException) {
    const name = e.name
    const message = e.message
    if (name === 'AbortError') {
      return `${Message.GET_USER_MEDIA_ABORT_ERROR} (name: ${name}, message: ${message})`
    } else if (name === 'NotAllowedError') {
      return `${Message.GET_USER_MEDIA_NOT_ALLOWED_ERROR} (name: ${name}, message: ${message})`
    } else if (name === 'NotFoundError') {
      return `${Message.GET_USER_MEDIA_NOT_FOUND_ERROR} (name: ${name}, message: ${message})`
    } else if (name === 'NotReadableError') {
      return `${Message.GET_USER_MEDIA_NOT_READABLE_ERROR} (name: ${name}, message: ${message})`
    } else if (name === 'OverconstrainedError') {
      return `${Message.GET_USER_MEDIA_OVERCONSTRAINED_ERROR} (name: ${name}, message: ${message})`
    } else if (name === 'SecurityError') {
      return `${Message.GET_USER_MEDIA_SECURITY_ERROR} (name: ${name}, message: ${message})`
    } else {
      return `${Message.GET_USER_MEDIA_DOMEXCEPTION_UNEXPECTED_ERROR} (name: ${name}, message: ${message})`
    }
  } else if (e instanceof TypeError) {
    const message = e.message
    return `${Message.GET_USER_MEDIA_TYPE_ERROR} (message: ${message})`
  } else {
    return `${Message.GET_USER_MEDIA_UNEXPECTED_ERROR}`
  }
}
