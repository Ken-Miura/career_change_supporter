import { Message } from '@/util/Message'
import { LocalAudioStream, LocalP2PRoomMember, P2PRoom, SkyWayContext, SkyWayRoom } from '@skyway-sdk/room'
import { SkyWayOriginatedError } from './SkyWayOriginatedError'

export class SkyWayAudioMeetingRoom {
  private context: SkyWayContext | null
  private room: P2PRoom | null
  private member: LocalP2PRoomMember | null
  private localAudioStream: LocalAudioStream | null
  private initilized: boolean
  private closed: boolean

  constructor () {
    this.context = null
    this.room = null
    this.member = null
    this.localAudioStream = null
    this.initilized = false
    this.closed = false
  }

  public async init (token: string, roomName: string, memberName: string, audioMediaStreamTrack: MediaStreamTrack) {
    if (this.initilized) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_AUDIO_MEETING_ROOM_HAS_ALREADY_BEEN_INITILIZED}`)
    }
    if (this.closed) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_AUDIO_MEETING_ROOM_HAS_ALREADY_BEEN_CLOSED}`)
    }
    // TODO: SkyWayに関する適切なエラーハンドリング、エラーメッセージを定義する
    try {
      this.context = await SkyWayContext.Create(token)
    } catch (e) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_FAILED_TO_CREATE_CONTEXT}: ${e}`)
    }
    if (!this.context) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_NO_CONTEXT_FOUND}`)
    }

    try {
      this.room = await SkyWayRoom.FindOrCreate(this.context, {
        type: 'p2p',
        name: roomName
      })
    } catch (e) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_FAILED_TO_FIND_OR_CREATE_ROOM}: ${e}`)
    }
    if (!this.room) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_NO_ROOM_FOUND}`)
    }

    try {
      this.member = await this.room.join({ name: memberName })
    } catch (e) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_FAILED_TO_CREATE_MEMBER}: ${e}`)
    }
    if (!this.member) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_NO_MEMBER_FOUND}`)
    }

    try {
      this.localAudioStream = new LocalAudioStream(audioMediaStreamTrack)
    } catch (e) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_FAILED_TO_CREATE_LOCAL_AUDIO_STREAM}: ${e}`)
    }
    if (!this.localAudioStream) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_NO_LOCAL_AUDIO_STREAM_FOUND}`)
    }
    this.initilized = true
  }

  public getContext (): SkyWayContext {
    this.ensureInitializedAndNotClosed()
    if (!this.context) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_NO_CONTEXT_FOUND}`)
    }
    return this.context
  }

  public getRoom (): P2PRoom {
    this.ensureInitializedAndNotClosed()
    if (!this.room) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_NO_ROOM_FOUND}`)
    }
    return this.room
  }

  public getMember (): LocalP2PRoomMember {
    this.ensureInitializedAndNotClosed()
    if (!this.member) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_NO_MEMBER_FOUND}`)
    }
    return this.member
  }

  public getLocalAudioStream (): LocalAudioStream {
    this.ensureInitializedAndNotClosed()
    if (!this.localAudioStream) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_NO_LOCAL_AUDIO_STREAM_FOUND}`)
    }
    return this.localAudioStream
  }

  ensureInitializedAndNotClosed () {
    if (!this.initilized) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_AUDIO_MEETING_ROOM_HAS_NOT_BEEN_INITILIZED_YET}`)
    }
    if (this.closed) {
      throw new SkyWayOriginatedError(`${Message.SKY_WAY_AUDIO_MEETING_ROOM_HAS_ALREADY_BEEN_CLOSED}`)
    }
  }

  public async close () {
    try {
      if (this.member) {
        await this.member.leave()
        this.member = null
      }
    } catch (e) {
      console.error(e)
    }
    try {
      if (this.localAudioStream) {
        this.localAudioStream.release()
        this.localAudioStream = null
      }
    } catch (e) {
      console.error(e)
    }
    try {
      if (this.room) {
        // roomに他のメンバーが残っている状態なのでcloseは呼ばない
        // 自身のリソースのみを開放するためにdisposeを使う
        await this.room.dispose()
        this.room = null
      }
    } catch (e) {
      console.error(e)
    }
    try {
      if (this.context) {
        this.context.dispose()
        this.context = null
      }
    } catch (e) {
      console.error(e)
    }
    this.closed = true
  }
}
