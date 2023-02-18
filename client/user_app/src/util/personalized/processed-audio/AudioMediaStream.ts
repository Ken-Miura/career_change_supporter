export async function getAudioMediaStream (): Promise<MediaStream> {
  const mediaStream = await window.navigator.mediaDevices
    .getUserMedia({
      audio: true,
      video: false
    })
  return mediaStream
}
