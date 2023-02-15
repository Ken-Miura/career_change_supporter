import { Message } from '@/util/Message'
import { LocalAudioStream, LocalP2PRoomMember, P2PRoom, RoomPublication, SkyWayContext } from '@skyway-sdk/room'
import { ref } from 'vue'

export function useSetupSkyWay () {
  const skyWayErrorExists = ref(false)
  const skyWayErrorMessage = ref('')
  const remoteMediaStream = ref(null as MediaStream | null)

  const setupSkyWay = (context: SkyWayContext, room: P2PRoom, member: LocalP2PRoomMember, localAudioStream: LocalAudioStream) => {
    member.publish(localAudioStream)

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
          skyWayErrorExists.value = true
          skyWayErrorMessage.value = Message.UNEXPECTED_ERR // TODO: replace error message
      }
    }

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

      context.onFatalError.add(args => {
        skyWayErrorExists.value = true
        skyWayErrorMessage.value = `${Message.UNEXPECTED_ERR}: ${args}` // TODO: Add error handling
      })

      member.onFatalError.add(args => {
        skyWayErrorExists.value = true
        skyWayErrorMessage.value = `${Message.UNEXPECTED_ERR}: ${args}` // TODO: Add error handling
      })
    })
  }

  return {
    skyWayErrorExists,
    skyWayErrorMessage,
    remoteMediaStream,
    setupSkyWay
  }
}
