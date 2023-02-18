<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getConsultantSideInfoDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div>
        <div v-if="!consultantSideInfo" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <div class="mt-2 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
            <div class="m-4 text-xl grid grid-cols-6 justify-center items-center">
              <div class="col-span-5">私は音声入出力テストで使用中の環境に問題がないことを確認しました</div>
              <input v-model="audioTestDone" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
            </div>
          </div>
          <button v-bind:disabled="!audioTestDone" v-on:click="processGetConsultantSideInfo" class="mt-4 col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">相談を開始する</button>
          <div v-if="getConsultantSideInfoErr.exists">
            <AlertMessage class="mt-2" v-bind:message="getConsultantSideInfoErr.message"/>
          </div>
        </div>
        <div v-else class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <div v-if="mediaError.exists">
            <AlertMessage class="mt-2" v-bind:message="mediaError.message"/>
          </div>
          <div v-else-if="skyWayErrorExists">
            <AlertMessage class="mt-2" v-bind:message="skyWayErrorMessage"/>
          </div>
          <div v-else>
            <div v-if="remoteMediaStream" class="flex flex-col items-center w-full">
              <img class="w-full md:w-3/5" src="/consultant-side-consultation/user-silhouette.png" />
              <audio v-bind:srcObject.prop="remoteMediaStream" autoplay>
                <p class="mt-4 font-bold text-xl">使われているブラウザではサービスを利用できません。他のブラウザをお使い下さい。</p>
              </audio>
              <p class="mt-2 text-xl text-center">相手側の音声が聞き取りづらいとき、一度退室し、再度入室するよう相手に促して下さい。再入室後、相手音声の高さ（低さ）が変化します。</p>
            </div>
            <div v-else>
              <h3 class="font-bold text-2xl text-center">相手が入室するまでお待ち下さい。</h3>
            </div>
            <p class="mt-6 text-xl text-center">相談時間（１時間）が過ぎたタイミングで会議室は自動的に閉じられません（相談時間が過ぎてから、一定時間後に自動的に閉じられます）相談してから１時間が経過したとき、あなたの判断で退出し、相談を終了させて下さい</p>
          </div>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">相談相手の情報</h3>
        <div class="m-4 text-2xl grid grid-cols-3">
          <div class="mt-2 justify-self-start col-span-2">ユーザーID</div><div class="mt-2 justify-self-start col-span-1">{{ userAccountId }}</div>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <button v-on:click="leaveConsultationRoom" class="col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">退出する</button>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onUnmounted, reactive, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRoute, useRouter } from 'vue-router'
import { Message } from '@/util/Message'
import { useGetConsultantSideInfo } from '@/util/personalized/consultant-side-consultation/useGetConsultantSideInfo'
import { GetConsultantSideInfoResp } from '@/util/personalized/consultant-side-consultation/GetConsultantSideInfoResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { ConsultantSideInfo } from '@/util/personalized/consultant-side-consultation/ConsultantSideInfo'
import { LocalAudioStream, LocalP2PRoomMember, P2PRoom, SkyWayContext } from '@skyway-sdk/room'
import { getAudioMediaStream } from '@/util/personalized/processed-audio/AudioMediaStream'
import { createGetAudioMediaStreamErrMessage } from '@/util/personalized/processed-audio/AudioMediaStreamError'
import { generatePitchFactor } from '@/util/personalized/processed-audio/PitchFacter'
import { PARAM_PITCH_FACTOR, PHASE_VOCODER_PROCESSOR_MODULE_NAME } from '@/util/personalized/processed-audio/PhaseVocoderProcessorConst'
import { useSetupSkyWay } from '@/util/personalized/skyway/useSetUpSkyWay'
import { createSkyWayItem } from '@/util/personalized/skyway/CreateSkyWayItem'

export default defineComponent({
  name: 'ConsultantSideConsultationPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const route = useRoute()
    const consultationId = route.params.consultation_id as string
    const userAccountId = route.params.user_account_id as string
    const router = useRouter()

    const getConsultantSideInfoErr = reactive({
      exists: false,
      message: ''
    })
    const {
      getConsultantSideInfoDone,
      getConsultantSideInfoFunc
    } = useGetConsultantSideInfo()
    const audioTestDone = ref(false)
    const consultantSideInfo = ref(null as ConsultantSideInfo | null)

    const mediaError = reactive({
      exists: false,
      message: ''
    })
    let localStream = null as MediaStream | null
    let audioCtx = null as AudioContext | null
    let processedStream = null as MediaStream | null

    const {
      skyWayErrorExists,
      skyWayErrorMessage,
      remoteMediaStream,
      setupSkyWay
    } = useSetupSkyWay()
    let context = null as SkyWayContext | null
    let room = null as P2PRoom | null
    let member = null as LocalP2PRoomMember | null
    let localAudioStream = null as LocalAudioStream | null

    const releaseAllResources = async () => {
      try {
        if (member) {
          await member.leave()
          member = null
        }
      } catch (e) {
        console.error(e)
      }
      try {
        if (localAudioStream) {
          localAudioStream.release()
          localAudioStream = null
        }
      } catch (e) {
        console.error(e)
      }
      try {
        if (room) {
          // roomに他のメンバーが残っている状態なのでcloseは呼ばない
          // 自身のリソースのみを開放するためにdisposeを使う
          await room.dispose()
          room = null
        }
      } catch (e) {
        console.error(e)
      }
      try {
        if (context) {
          context.dispose()
          context = null
        }
      } catch (e) {
        console.error(e)
      }
      try {
        if (remoteMediaStream.value) {
          remoteMediaStream.value.getTracks().forEach(track => track.stop())
          remoteMediaStream.value = null
        }
      } catch (e) {
        console.error(e)
      }
      try {
        if (processedStream) {
          processedStream.getTracks().forEach(track => track.stop())
          processedStream = null
        }
      } catch (e) {
        console.error(e)
      }
      try {
        if (audioCtx) {
          await audioCtx.close()
          audioCtx = null
        }
      } catch (e) {
        console.error(e)
      }
      try {
        if (localStream) {
          localStream.getTracks().forEach(track => track.stop())
          localStream = null
        }
      } catch (e) {
        console.error(e)
      }
    }

    const processGetConsultantSideInfo = async () => {
      await releaseAllResources()
      try {
        const resp = await getConsultantSideInfoFunc(consultationId, audioTestDone.value)
        if (!(resp instanceof GetConsultantSideInfoResp)) {
          if (!(resp instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${resp}`)
          }
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          }
          getConsultantSideInfoErr.exists = true
          getConsultantSideInfoErr.message = createErrorMessage(resp.getApiError().getCode())
          return
        }
        consultantSideInfo.value = resp.getConsultantSideInfo()
        if (!consultantSideInfo.value) {
          getConsultantSideInfoErr.exists = true
          getConsultantSideInfoErr.message = Message.UNEXPECTED_ERR
          return
        }
      } catch (e) {
        getConsultantSideInfoErr.exists = true
        getConsultantSideInfoErr.message = `${Message.UNEXPECTED_ERR}: ${e}`
        return
      }

      try {
        try {
          localStream = await getAudioMediaStream()
        } catch (e) {
          mediaError.exists = true
          mediaError.message = createGetAudioMediaStreamErrMessage(e)
          await releaseAllResources()
          return
        }
        if (!localStream) {
          mediaError.exists = true
          mediaError.message = Message.FAILED_TO_GET_LOCAL_MEDIA_STREAM_ERROR_MESSAGE
          await releaseAllResources()
          return
        }

        try {
          audioCtx = new AudioContext()
        } catch (e) {
          mediaError.exists = true
          mediaError.message = Message.FAILED_TO_CREATE_AUDIO_CONTEXT_MESSAGE
          await releaseAllResources()
          return
        }
        if (!audioCtx) {
          mediaError.exists = true
          mediaError.message = Message.FAILED_TO_GET_AUDIO_CONTEXT_MESSAGE
          await releaseAllResources()
          return
        }
        const source = audioCtx.createMediaStreamSource(localStream)
        const moduleUrl = new URL('@/util/personalized/processed-audio/PhaseVocoderProcessor.worker.js', import.meta.url)
        try {
          await audioCtx.audioWorklet.addModule(moduleUrl)
        } catch (e) {
          mediaError.exists = true
          mediaError.message = `${Message.FAILED_TO_ADD_MODULE_MESSAGE}: ${e}`
          await releaseAllResources()
          return
        }
        const phaseVocoderProcessorNode = new AudioWorkletNode(audioCtx, PHASE_VOCODER_PROCESSOR_MODULE_NAME)
        const param = phaseVocoderProcessorNode.parameters.get(PARAM_PITCH_FACTOR)
        if (!param) {
          mediaError.exists = true
          mediaError.message = `${Message.NO_PARAM_PITCH_FACTOR_FOUND_MESSAGE}`
          await releaseAllResources()
          return
        }
        param.value = generatePitchFactor()
        const destNode = audioCtx.createMediaStreamDestination()
        source.connect(phaseVocoderProcessorNode)
        phaseVocoderProcessorNode.connect(destNode)

        processedStream = destNode.stream
        if (!processedStream) {
          mediaError.exists = true
          mediaError.message = Message.NO_PROCESSED_STREAM_FOUND_MESSAGE
          await releaseAllResources()
          return
        }
      } catch (e) {
        mediaError.exists = true
        mediaError.message = `${Message.UNEXPECTED_ERR}: ${e}`
        await releaseAllResources()
        return
      }

      try {
        const audioTrack = processedStream.getAudioTracks()[0]
        const skyWayItem = await createSkyWayItem(consultantSideInfo.value.token, consultantSideInfo.value.room_name, consultantSideInfo.value.member_name, audioTrack)
        context = skyWayItem.context
        room = skyWayItem.room
        member = skyWayItem.member
        localAudioStream = skyWayItem.localAudioStream
        setupSkyWay(skyWayItem.context, skyWayItem.room, skyWayItem.member, skyWayItem.localAudioStream)
      } catch (e) {
        skyWayErrorExists.value = true
        skyWayErrorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
        await releaseAllResources()
      }
    }

    onUnmounted(async () => {
      await releaseAllResources()
    })

    const leaveConsultationRoom = async () => {
      await router.push('/schedule')
    }

    return {
      getConsultantSideInfoErr,
      getConsultantSideInfoDone,
      audioTestDone,
      processGetConsultantSideInfo,
      consultantSideInfo,
      mediaError,
      skyWayErrorExists,
      skyWayErrorMessage,
      remoteMediaStream,
      leaveConsultationRoom,
      userAccountId
    }
  }
})
</script>
