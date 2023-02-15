import { LocalAudioStream, LocalP2PRoomMember, P2PRoom, SkyWayContext } from '@skyway-sdk/room'

export type RoomItem = {
  context: SkyWayContext,
  room: P2PRoom,
  member: LocalP2PRoomMember,
  localAudioStream: LocalAudioStream
}
