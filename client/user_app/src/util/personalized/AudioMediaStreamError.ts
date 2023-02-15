import { Message } from '@/util/Message'

// eslint-disable-next-line
export function createGetAudioMediaStreamErrMessage (e: any): string {
  if (e instanceof DOMException) {
    const name = e.name
    const message = e.message
    if (name === 'AbortError') {
      return `${Message.GET_USER_MEDIA_ABORT_ERROR_MESSAGE} (name: ${name}, message: ${message})`
    } else if (name === 'NotAllowedError') {
      return `${Message.GET_USER_MEDIA_NOT_ALLOWED_ERROR_MESSAGE} (name: ${name}, message: ${message})`
    } else if (name === 'NotFoundError') {
      return `${Message.GET_USER_MEDIA_NOT_FOUND_ERROR_MESSAGE} (name: ${name}, message: ${message})`
    } else if (name === 'NotReadableError') {
      return `${Message.GET_USER_MEDIA_NOT_READABLE_ERROR_MESSAGE} (name: ${name}, message: ${message})`
    } else if (name === 'OverconstrainedError') {
      return `${Message.GET_USER_MEDIA_OVERCONSTRAINED_ERROR_MESSAGE} (name: ${name}, message: ${message})`
    } else if (name === 'SecurityError') {
      return `${Message.GET_USER_MEDIA_SECURITY_ERROR_MESSAGE} (name: ${name}, message: ${message})`
    } else {
      return `${Message.GET_USER_MEDIA_DOMEXCEPTION_UNEXPECTED_ERROR_MESSAGE} (name: ${name}, message: ${message})`
    }
  } else if (e instanceof TypeError) {
    const message = e.message
    return `${Message.GET_USER_MEDIA_TYPE_ERROR_MESSAGE} (message: ${message})`
  } else {
    return `${Message.GET_USER_MEDIA_UNEXPECTED_ERROR_MESSAGE}`
  }
}
