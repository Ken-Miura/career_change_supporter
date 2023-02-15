import { LocalAudioStream, SkyWayContext, SkyWayRoom } from '@skyway-sdk/room'
import { RoomItem } from './RoomItem'

export async function createRoomItem (token: string, roomName: string, memberName: string, audioMediaStreamTrack: MediaStreamTrack): Promise<RoomItem> {
  let context = null
  try {
    context = await SkyWayContext.Create(token)
  } catch (e) {
    throw new Error(`TODO: Add message: ${e}`)
  }
  if (!context) {
    throw new Error('TODO: Add message')
  }

  let room = null
  try {
    room = await SkyWayRoom.FindOrCreate(context, {
      type: 'p2p',
      name: roomName
    })
  } catch (e) {
    throw new Error(`TODO: Add message: ${e}`)
  }
  if (!room) {
    throw new Error('TODO: Add message')
  }

  let member = null
  try {
    member = await room.join({ name: memberName })
  } catch (e) {
    throw new Error(`TODO: Add message: ${e}`)
  }
  if (!member) {
    throw new Error('TODO: Add message')
  }

  let localAudioStream = null
  try {
    localAudioStream = new LocalAudioStream(audioMediaStreamTrack)
  } catch (e) {
    throw new Error(`TODO: Add message: ${e}`)
  }
  if (!localAudioStream) {
    throw new Error('TODO: Add message')
  }

  return {
    context,
    room,
    member,
    localAudioStream
  } as RoomItem
}
