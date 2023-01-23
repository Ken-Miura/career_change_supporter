import Peer from 'skyway-js'

export function closeConsultationRoom (peer: Peer | null, localStream: MediaStream | null) {
  try {
    if (peer) {
      peer.destroy()
      peer = null
    }
  } catch (e) {
    console.error(e)
  }
  try {
    if (localStream) {
      localStream.getTracks().forEach(track => track.stop())
      localStream = null
    }
  } catch (e) {
    console.error(e)
  }
}
