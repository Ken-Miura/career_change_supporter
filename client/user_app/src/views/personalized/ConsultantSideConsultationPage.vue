<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getConsultantSideInfoDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div>
        <div v-if="!consultantSideInfo" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
          <div class="mt-2 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
            <div class="m-4 text-lg lg:text-xl grid grid-cols-6 justify-center items-center">
              <div class="col-span-5">私は音声入出力テストで使用中の環境に問題がないことを確認しました</div>
              <input v-model="audioTestDone" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
            </div>
          </div>
          <button v-bind:disabled="!audioTestDone" v-on:click="processGetConsultantSideInfo" class="mt-4 col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">相談を開始する</button>
          <div v-if="getConsultantSideInfoErrMessage">
            <AlertMessage class="mt-2" v-bind:message="getConsultantSideInfoErrMessage"/>
          </div>
        </div>
        <div v-else class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
          <div v-if="audioErrorMessage">
            <AlertMessage class="mt-2" v-bind:message="audioErrorMessage"/>
          </div>
          <div v-else-if="skyWayErrorMessage">
            <AlertMessage class="mt-2" v-bind:message="skyWayErrorMessage"/>
          </div>
          <div v-else>
            <div v-if="remoteMediaStream" class="flex flex-col items-center w-full">
              <img class="w-full lg:w-3/5" src="/consultant-side-consultation/user-silhouette.png" />
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
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-xl lg:text-2xl">相談相手の情報</h3>
        <div class="m-4 text-xl lg:text-2xl grid grid-cols-2">
          <div class="mt-2 justify-self-start col-span-1">ユーザーID</div><div class="mt-2 justify-self-start col-span-1">{{ userAccountId }}</div>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <button v-on:click="leaveConsultationRoom" class="col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">退出する</button>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onUnmounted, ref } from 'vue'
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
import { useSetupSkyWay } from '@/util/personalized/skyway/useSetUpSkyWay'
import { ProcessedAudio } from '@/util/personalized/processed-audio/ProcessedAudio'
import { ProcessedAudioError } from '@/util/personalized/processed-audio/ProcessedAudioError'
import { SkyWayOriginatedError } from '@/util/personalized/skyway/SkyWayOriginatedError'
import { SkyWayAudioMeetingRoom } from '@/util/personalized/skyway/SkyWayAudioMeetingRoom'

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

    const getConsultantSideInfoErrMessage = ref(null as string | null)
    const {
      getConsultantSideInfoDone,
      getConsultantSideInfoFunc
    } = useGetConsultantSideInfo()
    const audioTestDone = ref(false)
    const consultantSideInfo = ref(null as ConsultantSideInfo | null)

    const audioErrorMessage = ref(null as string | null)
    let processedAudio: ProcessedAudio | null

    const {
      skyWayErrorMessage,
      remoteMediaStream,
      setupSkyWay
    } = useSetupSkyWay()
    let audioMeetingRoom = null as SkyWayAudioMeetingRoom | null

    const releaseAllResources = async () => {
      if (audioMeetingRoom) {
        await audioMeetingRoom.close()
        audioMeetingRoom = null
      }
      try {
        if (remoteMediaStream.value) {
          remoteMediaStream.value.getTracks().forEach(track => track.stop())
          remoteMediaStream.value = null
        }
      } catch (e) {
        console.error(e)
      }
      if (processedAudio) {
        await processedAudio.close()
        processedAudio = null
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
          getConsultantSideInfoErrMessage.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
        consultantSideInfo.value = resp.getConsultantSideInfo()
        if (!consultantSideInfo.value) {
          getConsultantSideInfoErrMessage.value = Message.UNEXPECTED_ERR
          return
        }
      } catch (e) {
        getConsultantSideInfoErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
        return
      }

      try {
        const pa = new ProcessedAudio()
        processedAudio = pa
        await pa.init()

        const amr = new SkyWayAudioMeetingRoom()
        audioMeetingRoom = amr
        await amr.init(consultantSideInfo.value.token, consultantSideInfo.value.room_name, consultantSideInfo.value.member_name, pa.getAudioMediaStreamTrack())
        setupSkyWay(amr)
      } catch (e) {
        if (e instanceof ProcessedAudioError) {
          audioErrorMessage.value = `${e.message}`
        } else if (e instanceof SkyWayOriginatedError) {
          skyWayErrorMessage.value = `${e.message}`
        } else {
          audioErrorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
        }
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
      getConsultantSideInfoErrMessage,
      getConsultantSideInfoDone,
      audioTestDone,
      processGetConsultantSideInfo,
      consultantSideInfo,
      audioErrorMessage,
      skyWayErrorMessage,
      remoteMediaStream,
      leaveConsultationRoom,
      userAccountId
    }
  }
})
</script>
