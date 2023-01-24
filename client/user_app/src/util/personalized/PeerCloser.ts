import Peer from 'skyway-js'

export function closePeer (peer: Peer | null) {
  try {
    if (peer) {
      peer.destroy()
      peer = null
    }
  } catch (e) {
    console.error(e)
  }
}
