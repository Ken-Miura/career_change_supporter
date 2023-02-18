import { Message } from '@/util/Message'
import { PARAM_PITCH_FACTOR, PHASE_VOCODER_PROCESSOR_MODULE_NAME } from './PhaseVocoderProcessorConst'
import { ProcessedAudioError } from './ProcessedAudioError'
import { generatePitchFactor } from './PitchFacter'
import { getAudioMediaStream } from './AudioMediaStream'
import { createGetAudioMediaStreamErrMessage } from './AudioMediaStreamError'

async function createProceesedAudio (audioCtx: AudioContext, localStream: MediaStream): Promise<AudioWorkletNode> {
  const source = audioCtx.createMediaStreamSource(localStream)
  const moduleUrl = new URL('@/util/personalized/processed-audio/PhaseVocoderProcessor.worker.js', import.meta.url)
  try {
    await audioCtx.audioWorklet.addModule(moduleUrl)
  } catch (e) {
    throw new ProcessedAudioError(`${Message.FAILED_TO_ADD_MODULE}: ${e}`)
  }
  const phaseVocoderProcessorNode = new AudioWorkletNode(audioCtx, PHASE_VOCODER_PROCESSOR_MODULE_NAME)
  const param = phaseVocoderProcessorNode.parameters.get(PARAM_PITCH_FACTOR)
  if (!param) {
    throw new ProcessedAudioError(`${Message.NO_PARAM_PITCH_FACTOR_FOUND}`)
  }
  param.value = generatePitchFactor()
  source.connect(phaseVocoderProcessorNode)
  return phaseVocoderProcessorNode
}

export class ProcessedAudio {
  private audioCtx: AudioContext | null
  private localStream: MediaStream | null
  private processedStream: MediaStream | null
  private initilized: boolean
  private closed: boolean

  constructor () {
    this.audioCtx = null
    this.localStream = null
    this.processedStream = null
    this.initilized = false
    this.closed = false
  }

  /**
   * ProcessedAudioを初期化する
   *
   * 内部でAudioContextをnewして生成するので、必ずユーザーとの対話イベントの中で呼び出す
   */
  public async init () {
    if (this.initilized) {
      throw new ProcessedAudioError(`${Message.PROCESSED_AUDIO_HAS_ALREADY_BEEN_INITILIZED}`)
    }
    if (this.closed) {
      throw new ProcessedAudioError(`${Message.PROCESSED_AUDIO_HAS_ALREADY_BEEN_CLOSED}`)
    }
    try {
      this.localStream = await getAudioMediaStream()
    } catch (e) {
      const message = createGetAudioMediaStreamErrMessage(e)
      throw new ProcessedAudioError(`${message}`)
    }
    if (!this.localStream) {
      throw new ProcessedAudioError(`${Message.FAILED_TO_GET_LOCAL_MEDIA_STREAM_ERROR}`)
    }

    try {
      this.audioCtx = new AudioContext()
    } catch (e) {
      throw new ProcessedAudioError(`${Message.FAILED_TO_CREATE_AUDIO_CONTEXT}: ${e}`)
    }
    if (!this.audioCtx) {
      throw new ProcessedAudioError(`${Message.FAILED_TO_GET_AUDIO_CONTEXT}`)
    }

    const phaseVocoderProcessorNode = await createProceesedAudio(this.audioCtx, this.localStream)
    const destNode = this.audioCtx.createMediaStreamDestination()
    phaseVocoderProcessorNode.connect(destNode)
    this.processedStream = destNode.stream
    this.initilized = true
  }

  public getAudioMediaStreamTrack (): MediaStreamTrack {
    if (!this.initilized) {
      throw new ProcessedAudioError(`${Message.PROCESSED_AUDIO_HAS_NOT_BEEN_INITILIZED_YET}`)
    }
    if (this.closed) {
      throw new ProcessedAudioError(`${Message.PROCESSED_AUDIO_HAS_ALREADY_BEEN_CLOSED}`)
    }
    if (!this.processedStream) {
      throw new ProcessedAudioError(`${Message.NO_PROCESSED_STREAM_FOUND}`)
    }
    return this.processedStream.getAudioTracks()[0]
  }

  public async close () {
    try {
      if (this.processedStream) {
        this.processedStream.getTracks().forEach(track => track.stop())
        this.processedStream = null
      }
    } catch (e) {
      console.error(e)
    }
    try {
      if (this.audioCtx) {
        await this.audioCtx.close()
        this.audioCtx = null
      }
    } catch (e) {
      console.error(e)
    }
    try {
      if (this.localStream) {
        this.localStream.getTracks().forEach(track => track.stop())
        this.localStream = null
      }
    } catch (e) {
      console.error(e)
    }
    this.closed = true
  }
}

export class ProcessedAudioConnectedWithSpeaker {
  private audioCtx: AudioContext | null
  private localStream: MediaStream | null
  private initilized: boolean
  private closed: boolean

  constructor () {
    this.audioCtx = null
    this.localStream = null
    this.initilized = false
    this.closed = false
  }

  /**
   * ProcessedAudioConnectedWithSpeakerを初期化する
   *
   * 内部でAudioContextをnewして生成するので、必ずユーザーとの対話イベントの中で呼び出す
   */
  public async init () {
    if (this.initilized) {
      throw new ProcessedAudioError(`${Message.PROCESSED_AUDIO_CONNECTED_WITH_SPEAKER_HAS_ALREADY_BEEN_INITILIZED}`)
    }
    if (this.closed) {
      throw new ProcessedAudioError(`${Message.PROCESSED_AUDIO_CONNECTED_WITH_SPEAKER_HAS_ALREADY_BEEN_CLOSED}`)
    }
    try {
      this.localStream = await getAudioMediaStream()
    } catch (e) {
      const message = createGetAudioMediaStreamErrMessage(e)
      throw new ProcessedAudioError(`${message}`)
    }
    if (!this.localStream) {
      throw new ProcessedAudioError(`${Message.FAILED_TO_GET_LOCAL_MEDIA_STREAM_ERROR}`)
    }

    try {
      this.audioCtx = new AudioContext()
    } catch (e) {
      throw new ProcessedAudioError(`${Message.FAILED_TO_CREATE_AUDIO_CONTEXT}: ${e}`)
    }
    if (!this.audioCtx) {
      throw new ProcessedAudioError(`${Message.FAILED_TO_GET_AUDIO_CONTEXT}`)
    }

    const phaseVocoderProcessorNode = await createProceesedAudio(this.audioCtx, this.localStream)
    phaseVocoderProcessorNode.connect(this.audioCtx.destination)
    this.initilized = true
  }

  public async close () {
    try {
      if (this.audioCtx) {
        await this.audioCtx.close()
        this.audioCtx = null
      }
    } catch (e) {
      console.error(e)
    }
    try {
      if (this.localStream) {
        this.localStream.getTracks().forEach(track => track.stop())
        this.localStream = null
      }
    } catch (e) {
      console.error(e)
    }
    this.closed = true
  }
}
