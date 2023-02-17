import { ref } from 'vue'
import { Message } from '../Message'
import { generatePitchFactor } from './audio-test/PitchFacter'
import { getAudioMediaStream } from './AudioMediaStream'
import { createGetAudioMediaStreamErrMessage } from './AudioMediaStreamError'
import { PARAM_PITCH_FACTOR, PHASE_VOCODER_PROCESSOR_MODULE_NAME } from './PhaseVocoderProcessorConst'

export function useSetupProcessedAudio () {
  const audioErrorMessage = ref(null as string | null)
  const localStream = ref(null as MediaStream | null)
  const audioCtx = ref(null as AudioContext | null)
  const processedStream = ref(null as MediaStream | null)

  const releaseAudioResouces = async () => {
    try {
      if (processedStream.value) {
        processedStream.value.getTracks().forEach(track => track.stop())
        processedStream.value = null
      }
    } catch (e) {
      console.error(e)
    }
    try {
      if (audioCtx.value) {
        await audioCtx.value.close()
        audioCtx.value = null
      }
    } catch (e) {
      console.error(e)
    }
    try {
      if (localStream.value) {
        localStream.value.getTracks().forEach(track => track.stop())
        localStream.value = null
      }
    } catch (e) {
      console.error(e)
    }
  }

  const setup = async (connectSpeaker: boolean) => {
    try {
      localStream.value = await getAudioMediaStream()
    } catch (e) {
      audioErrorMessage.value = createGetAudioMediaStreamErrMessage(e)
      await releaseAudioResouces()
      return
    }
    if (!localStream.value) {
      audioErrorMessage.value = Message.FAILED_TO_GET_LOCAL_MEDIA_STREAM_ERROR_MESSAGE
      await releaseAudioResouces()
      return
    }

    try {
      audioCtx.value = new AudioContext()
    } catch (e) {
      audioErrorMessage.value = Message.FAILED_TO_CREATE_AUDIO_CONTEXT_MESSAGE
      await releaseAudioResouces()
      return
    }
    if (!audioCtx.value) {
      audioErrorMessage.value = Message.FAILED_TO_GET_AUDIO_CONTEXT_MESSAGE
      await releaseAudioResouces()
      return
    }
    const source = audioCtx.value.createMediaStreamSource(localStream.value)
    const moduleUrl = new URL('@/util/personalized/PhaseVocoderProcessor.worker.js', import.meta.url)
    try {
      await audioCtx.value.audioWorklet.addModule(moduleUrl)
    } catch (e) {
      audioErrorMessage.value = `${Message.FAILED_TO_ADD_MODULE_MESSAGE}: ${e}`
      await releaseAudioResouces()
      return
    }
    const phaseVocoderProcessorNode = new AudioWorkletNode(audioCtx.value, PHASE_VOCODER_PROCESSOR_MODULE_NAME)
    const param = phaseVocoderProcessorNode.parameters.get(PARAM_PITCH_FACTOR)
    if (!param) {
      audioErrorMessage.value = `${Message.NO_PARAM_PITCH_FACTOR_FOUND_MESSAGE}`
      await releaseAudioResouces()
      return
    }
    param.value = generatePitchFactor()
    source.connect(phaseVocoderProcessorNode)
    if (connectSpeaker) {
      phaseVocoderProcessorNode.connect(audioCtx.value.destination)
    } else {
      const destNode = audioCtx.value.createMediaStreamDestination()
      phaseVocoderProcessorNode.connect(destNode)
      processedStream.value = destNode.stream
    }
  }

  const setupProcessedAudioForTest = async () => {
    setup(true)
  }

  const setupProcessedAudio = async () => {
    setup(false)
  }

  return {
    audioErrorMessage,
    processedStream,
    releaseAudioResouces,
    setupProcessedAudioForTest,
    setupProcessedAudio
  }
}
