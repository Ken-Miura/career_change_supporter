<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!(getUserSideInfoDone && getConsultantDetailDone)" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="peerError.exists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="peerError.message"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <div v-if="remoteMediaStream" class="flex flex-col justify-center items-center self-center w-full md:w-3/5">
            <img class="w-full md:w-4/5 self-center" src="/user-side-consultation/consultant-silhouette.png" />
            <audio v-bind:srcObject.prop="remoteMediaStream" autoplay>
              <p class="mt-4 font-bold text-xl">使われているブラウザではサービスを利用できません。他のブラウザをお使い下さい。</p>
            </audio>
          </div>
          <div v-else>
            <h3 class="font-bold text-2xl text-center">相手が入室するまでお待ち下さい。</h3>
            <h3 class="font-bold text-2xl text-center">相手との接続が切断された場合、一度退出し、再度入室して下さい。</h3>
          </div>
          <button v-on:click="processGetUserSideInfo" class="col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">テスト</button>
        </div>
      </div>
      <div v-if="error.exists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="error.message"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 data-test="consultant-detail-label" class="font-bold text-2xl">相談相手の情報</h3>
          <div v-if="consultantDetail !== null" class="m-4 text-2xl grid grid-cols-3">
            <div data-test="consultant-id-label" class="mt-2 justify-self-start col-span-2">コンサルタントID</div><div data-test="consultant-id-value" class="mt-2 justify-self-start col-span-1">{{ consultantDetail.consultant_id }}</div>
            <div data-test="career-label" class="mt-5 justify-self-start col-span-3 font-bold text-2xl">職務経歴</div>
            <div class="mt-2 justify-self-start col-span-3 flex flex-col justify-center w-full">
              <ul>
                <li v-for="(consultantCareerDetail, index) in consultantDetail.careers" v-bind:key="consultantCareerDetail.counsultant_career_detail_id">
                  <div v-bind:data-test="'career-detail-' + index" class="mt-2">
                    <div data-test="career-detail-label" class="bg-gray-600 text-white font-bold text-xl rounded-t px-4 py-2">職務経歴{{ index + 1 }}</div>
                    <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                      <div data-test="company-name-label" class="mt-2 justify-self-start col-span-1">勤務先名称</div><div data-test="company-name-value" class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.company_name }}</div>
                      <div data-test="department-name-label" v-if="consultantCareerDetail.department_name" class="mt-2 justify-self-start col-span-1">部署名</div><div data-test="department-name-value" v-if="consultantCareerDetail.department_name" class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.department_name }}</div>
                      <div data-test="office-label" v-if="consultantCareerDetail.office" class="mt-2 justify-self-start col-span-1">勤務地</div><div data-test="office-value" v-if="consultantCareerDetail.office" class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.office }}</div>
                      <div data-test="years-of-service-label" class="mt-2 justify-self-start col-span-1">在籍年数</div><div data-test="years-of-service-value" class="mt-2 justify-self-start col-span-2">{{ convertYearsOfServiceValue(consultantCareerDetail.years_of_service) }}</div>
                      <div data-test="employed-label" class="mt-2 justify-self-start col-span-1">在籍の有無</div><div data-test="employed-value" class="mt-2 justify-self-start col-span-2">{{ convertEmployedValue(consultantCareerDetail.employed) }}</div>
                      <div data-test="contract-type-label" class="mt-2 justify-self-start col-span-1">雇用形態</div><div data-test="contract-type-value" class="mt-2 justify-self-start col-span-2">{{ convertContractTypeValue(consultantCareerDetail.contract_type) }}</div>
                      <div data-test="profession-label" v-if="consultantCareerDetail.profession" class="mt-2 justify-self-start col-span-1">職種</div><div data-test="profession-value" v-if="consultantCareerDetail.profession" class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.profession }}</div>
                      <div data-test="annual-income-in-man-yen-label" v-if="consultantCareerDetail.annual_income_in_man_yen" class="mt-2 justify-self-start col-span-1">年収</div><div data-test="annual-income-in-man-yen-value" v-if="consultantCareerDetail.annual_income_in_man_yen" class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.annual_income_in_man_yen }}万円</div>
                      <div data-test="is-manager-label" class="mt-2 justify-self-start col-span-1">管理職区分</div><div data-test="is-manager-value" class="mt-2 justify-self-start col-span-2">{{ convertIsManagerValue(consultantCareerDetail.is_manager) }}</div>
                      <div data-test="position-name-label" v-if="consultantCareerDetail.position_name" class="mt-2 justify-self-start col-span-1">職位</div><div data-test="position-name-value" v-if="consultantCareerDetail.position_name" class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.position_name }}</div>
                      <div data-test="is-new-graduate-label" class="mt-2 justify-self-start col-span-1">入社区分</div><div data-test="is-new-graduate-value" class="mt-2 justify-self-start col-span-2">{{ convertIsNewGraduateValue(consultantCareerDetail.is_new_graduate) }}</div>
                      <div data-test="note-label" v-if="consultantCareerDetail.note" class="mt-2 justify-self-start col-span-1">備考</div><div data-test="note-value" v-if="consultantCareerDetail.note" class="mt-2 justify-self-start col-span-2 whitespace-pre-wrap">{{ consultantCareerDetail.note }}</div>
                    </div>
                  </div>
                </li>
              </ul>
            </div>
          </div>
          <p v-else class="m-4 text-xl">相談相手の情報を取得出来ませんでした。</p>
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
import { defineComponent, onMounted, onUnmounted, reactive, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRoute, useRouter } from 'vue-router'
import { useGetUserSideInfo } from '@/util/personalized/user-side-consultation/useGetUserSideInfo'
import { closeMediaStream } from '@/util/personalized/MediaStreamCloser'
import { Message } from '@/util/Message'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { createGetAudioMediaStreamErrMessage, getAudioMediaStream } from '@/util/personalized/AudioMediaStream'
import { useGetConsultantDetail } from '@/util/personalized/consultant-detail/useGetConsultantDetail'
import { GetConsultantDetailResp } from '@/util/personalized/consultant-detail/GetConsultantDetailResp'
import { ConsultantDetail } from '@/util/personalized/consultant-detail/ConsultantDetail'
import { convertYearsOfServiceValue, convertEmployedValue, convertContractTypeValue, convertIsManagerValue, convertIsNewGraduateValue } from '@/util/personalized/ConsultantDetailConverter'
import {
  LocalAudioStream,
  nowInSec,
  RemoteVideoStream,
  SkyWayAuthToken,
  SkyWayContext,
  SkyWayRoom,
  uuidV4
} from '@skyway-sdk/room'

export default defineComponent({
  name: 'UserSideConsultationPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const appId = 'TODO'
    const secret = 'TODO'
    const token = new SkyWayAuthToken({
      jti: uuidV4(),
      iat: nowInSec(),
      exp: nowInSec() + 60 * 60 * 24,
      scope: {
        app: {
          id: appId,
          turn: true,
          actions: ['read'],
          channels: [
            {
              id: '*',
              name: '*',
              actions: ['write'],
              members: [
                {
                  id: '*',
                  name: '*',
                  actions: ['write'],
                  publication: {
                    actions: ['write']
                  },
                  subscription: {
                    actions: ['write']
                  }
                }
              ],
              sfuBots: [
                {
                  actions: ['write'],
                  forwardings: [
                    {
                      actions: ['write']
                    }
                  ]
                }
              ]
            }
          ]
        }
      }
    }).encode(secret)
    const roomName = '6ebb6af49c654d10a41babbe60251baf'

    const route = useRoute()
    const consultationId = route.params.consultation_id as string
    const consultantId = route.params.consultant_id as string

    const {
      getUserSideInfoDone,
      getUserSideInfoFunc
    } = useGetUserSideInfo()

    let localStream = null as MediaStream | null
    let audioCtx = null as AudioContext | null
    const peerError = reactive({
      exists: false,
      message: ''
    })
    const remoteMediaStream = ref(null as MediaStream | null)
    let processedLocalStream = null as MediaStream | null

    const error = reactive({
      exists: false,
      message: ''
    })
    const consultantDetail = ref(null as ConsultantDetail | null)
    const {
      getConsultantDetailDone,
      getConsultantDetailFunc
    } = useGetConsultantDetail()

    const router = useRouter()

    const releaseAllResources = async () => {
      closeMediaStream(localStream)
      closeMediaStream(processedLocalStream)
      closeMediaStream(remoteMediaStream.value)
      if (audioCtx) {
        await audioCtx.close()
      }
    }

    const processGetUserSideInfo = async () => {
      await releaseAllResources()
      try {
        try {
          localStream = await getAudioMediaStream()
        } catch (e) {
          peerError.exists = true
          peerError.message = createGetAudioMediaStreamErrMessage(e)
          return
        }
        if (!localStream) {
          peerError.exists = true
          peerError.message = Message.FAILED_TO_GET_LOCAL_MEDIA_STREAM_ERROR_MESSAGE
          return
        }

        audioCtx = new AudioContext()
        if (!audioCtx) {
          peerError.exists = true
          peerError.message = Message.FAILED_TO_GET_LOCAL_MEDIA_STREAM_ERROR_MESSAGE
          return
        }
        const source = audioCtx.createMediaStreamSource(localStream)
        const u = new URL('@/util/personalized/PhaseVocoderProcessor.worker.js', import.meta.url)
        try {
          await audioCtx.audioWorklet.addModule(u)
        } catch (e) {
          console.error(`failed to call addModule: ${e}`)
          return
        }
        const phaseVocoderProcessorNode = new AudioWorkletNode(audioCtx, 'phase-vocoder-processor')
        const param = phaseVocoderProcessorNode.parameters.get('pitchFactor')
        if (param) {
          param.value = 1.25 * 1 / 1
        }

        const dest = audioCtx.createMediaStreamDestination()
        processedLocalStream = dest.stream
        if (!processedLocalStream) {
          peerError.exists = true
          peerError.message = Message.FAILED_TO_GET_LOCAL_MEDIA_STREAM_ERROR_MESSAGE
          return
        }

        source.connect(phaseVocoderProcessorNode)
        phaseVocoderProcessorNode.connect(dest)

        const context = await SkyWayContext.Create(token)
        const room = await SkyWayRoom.FindOrCreate(context, {
          type: 'p2p',
          name: roomName
        })

        const me = await room.join()
        const audioTracks = processedLocalStream.getAudioTracks()
        const localAudioStream = new LocalAudioStream(audioTracks[0].clone())
        await me.publish(localAudioStream)
        // eslint-disable-next-line
        const subscribeAndAttach = async (publication: any) => {
          if (publication.publisher.id === me.id) {
            return
          }
          const { stream } = await me.subscribe<RemoteVideoStream>(
            publication.id
          )
          switch (stream.track.kind) {
            case 'video':
              console.error('video')
              break
            case 'audio':
              remoteMediaStream.value = new MediaStream([stream.track])
              break
            default:
              console.error('default')
          }
        }
        room.publications.forEach(subscribeAndAttach)
        room.onStreamPublished.add((e) => subscribeAndAttach(e.publication))
      } catch (e) {
        peerError.exists = true
        peerError.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const processGetConsultantDetail = async () => {
      try {
        const resp = await getConsultantDetailFunc(consultantId)
        if (!(resp instanceof GetConsultantDetailResp)) {
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
          error.exists = true
          error.message = createErrorMessage(resp.getApiError().getCode())
          return
        }
        consultantDetail.value = resp.getConsultantDetail()
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    onMounted(async () => {
      // await processGetUserSideInfo()
      await processGetConsultantDetail()
    })

    onUnmounted(async () => {
      await releaseAllResources()
    })

    const leaveConsultationRoom = async () => {
      await router.push('/schedule')
    }

    return {
      getUserSideInfoDone,
      peerError,
      remoteMediaStream,
      leaveConsultationRoom,
      getConsultantDetailDone,
      error,
      consultantDetail,
      convertYearsOfServiceValue,
      convertEmployedValue,
      convertContractTypeValue,
      convertIsManagerValue,
      convertIsNewGraduateValue,
      processGetUserSideInfo
    }
  }
})
</script>
