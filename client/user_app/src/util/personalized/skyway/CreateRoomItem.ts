import { Message } from '@/util/Message'
import { LocalAudioStream, SkyWayContext, SkyWayRoom } from '@skyway-sdk/room'
import { RoomItem } from './RoomItem'

export async function createRoomItem (token: string, roomName: string, memberName: string, audioMediaStreamTrack: MediaStreamTrack): Promise<RoomItem> {
  let context = null
  try {
    context = await SkyWayContext.Create(token)
  } catch (e) {
    throw new Error(`${Message.SKY_WAY_FAILED_TO_CREATE_CONTEXT_MESSAGE}: ${e}`)
  }
  if (!context) {
    throw new Error(`${Message.SKY_WAY_NO_CONTEXT_FOUND_MESSAGE}`)
  }

  let room = null
  try {
    room = await SkyWayRoom.FindOrCreate(context, {
      type: 'p2p',
      name: roomName
    })
  } catch (e) {
    throw new Error(`${Message.SKY_WAY_FAILED_TO_CREATE_ROOM_MESSAGE}: ${e}`)
  }
  if (!room) {
    throw new Error(`${Message.SKY_WAY_NO_ROOM_FOUND_MESSAGE}`)
  }

  let member = null
  try {
    member = await room.join({ name: memberName })
  } catch (e) {
    throw new Error(`${Message.SKY_WAY_FAILED_TO_CREATE_MEMBER_MESSAGE}: ${e}`)
  }
  if (!member) {
    throw new Error(`${Message.SKY_WAY_NO_MEMBER_FOUND_MESSAGE}`)
  }

  let localAudioStream = null
  try {
    localAudioStream = new LocalAudioStream(audioMediaStreamTrack)
  } catch (e) {
    throw new Error(`${Message.SKY_WAY_FAILED_TO_CREATE_LOCAL_AUDIO_STREAM_MESSAGE}: ${e}`)
  }
  if (!localAudioStream) {
    throw new Error(`${Message.SKY_WAY_NO_LOCAL_AUDIO_STREAM_FOUND_MESSAGE}`)
  }

  return {
    context,
    room,
    member,
    localAudioStream
  } as RoomItem
}
