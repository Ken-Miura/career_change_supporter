import Peer, { PeerError } from 'skyway-js'
import { reactive, ref } from 'vue'
import { Message } from '../Message'

export function usePeerHandleRegister () {
  const peerError = reactive({
    exists: false,
    message: ''
  })
  const remoteMediaStream = ref(null as MediaStream | null)

  const createErrMessage = (errType: string, e: PeerError):string => {
    // TODO: Add errors
    return `${Message.UNEXPECTED_ERR}: ${e} ${errType}`
  }

  const registerErrorHandler = (peer: Peer) => {
    peer.on('error', e => {
      const errType = e.type
      // fetchPeerExistsを行わずにcallを行うので発生が予見されるエラー
      // そのため、特に何もしない（一度お互いに入室し、その後何らかの理由で再度入室することになった場合発生し得る）
      if (errType === 'peer-unavailable') {
        return
      }
      peerError.exists = true
      peerError.message = createErrMessage(errType, e)
    })
  }

  const registerReceiveCallHandler = (peer: Peer, localStream: MediaStream) => {
    peer.on('call', (mediaConnection) => {
      mediaConnection.answer(localStream)

      mediaConnection.on('stream', stream => {
        remoteMediaStream.value = stream
      })

      mediaConnection.once('close', () => {
        if (!remoteMediaStream.value) {
          return
        }
        remoteMediaStream.value.getTracks().forEach(track => track.stop())
        remoteMediaStream.value = null
      })
    })
  }

  const registerCallOnOpenHandler = (peer: Peer, localStream: MediaStream, remotePeerId: string) => {
    peer.on('open', () => {
      // fetchPeerExistsで事前に確認してから通信したほうが確実だが
      // rate limitが厳しすぎるので使わない
      const mediaConnection = peer.call(remotePeerId, localStream)

      mediaConnection.on('stream', stream => {
        remoteMediaStream.value = stream
      })

      mediaConnection.once('close', () => {
        if (!remoteMediaStream.value) {
          return
        }
        remoteMediaStream.value.getTracks().forEach(track => track.stop())
        remoteMediaStream.value = null
      })
    })
  }

  return {
    peerError,
    remoteMediaStream,
    registerErrorHandler,
    registerReceiveCallHandler,
    registerCallOnOpenHandler
  }
}
