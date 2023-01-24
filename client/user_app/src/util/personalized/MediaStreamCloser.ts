export function closeMediaStream (mediaStream: MediaStream | null) {
  try {
    if (mediaStream) {
      mediaStream.getTracks().forEach(track => track.stop())
      mediaStream = null
    }
  } catch (e) {
    console.error(e)
  }
}
