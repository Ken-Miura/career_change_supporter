import { Message } from '@/util/Message'
import { RoomPublication } from '@skyway-sdk/room'
import { ref } from 'vue'
import { SkyWayAudioMeetingRoom } from './SkyWayAudioMeetingRoom'

export function useSetupSkyWay () {
  const skyWayErrorMessage = ref(null as string | null)
  const remoteMediaStream = ref(null as MediaStream | null)

  const setupSkyWay = (audioMeetingRoom: SkyWayAudioMeetingRoom) => {
    audioMeetingRoom.getContext().onFatalError.add(args => {
      skyWayErrorMessage.value = `${Message.SKY_WAY_ROOM_ON_FATAL_ERROR}: ${args}`
    })

    const member = audioMeetingRoom.getMember()
    member.onFatalError.add(args => {
      skyWayErrorMessage.value = `${Message.SKY_WAY_MEMBER_ON_FATAL_ERROR}: ${args}`
    })
    member.publish(audioMeetingRoom.getLocalAudioStream())

    const subscribe = async (publication: RoomPublication) => {
      if (publication.publisher.id === member.id) {
        return
      }
      const { stream } = await member.subscribe(publication.id)
      switch (stream.contentType) {
        case 'audio':
          remoteMediaStream.value = new MediaStream([stream.track])
          break
        default:
          skyWayErrorMessage.value = Message.NON_AUDIO_STREAM_DETECTED
      }
    }

    const room = audioMeetingRoom.getRoom()
    room.publications.forEach(subscribe)
    room.onStreamPublished.add((e) => subscribe(e.publication))
    room.onMemberLeft.add(e => {
      if (member.id === e.member.id) {
        return
      }
      // 1対1の通話のため、自身以外がRoomから出たのであればRoomには誰もいない
      // そのため、映像は切断となる
      try {
        if (remoteMediaStream.value) {
          remoteMediaStream.value.getTracks().forEach(track => track.stop())
          remoteMediaStream.value = null
        }
      } catch (e) {
        console.error(e)
      }
    })
  }

  return {
    skyWayErrorMessage,
    remoteMediaStream,
    setupSkyWay
  }
}
