<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!(getUserSideInfoDone && getConsultantDetailDone)" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div>
        <div v-if="!userSideInfo" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <div class="mt-2 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
            <div class="m-4 text-xl grid grid-cols-6 justify-center items-center">
              <div class="col-span-5">私は音声入出力テストで使用中の環境に問題がないことを確認しました</div>
              <input v-model="audioTestDone" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
            </div>
          </div>
          <button v-bind:disabled="!audioTestDone" v-on:click="processGetUserSideInfo" class="mt-4 col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">相談を開始する</button>
          <div v-if="getUserSideInfoErrMessage">
            <AlertMessage class="mt-2" v-bind:message="getUserSideInfoErrMessage"/>
          </div>
        </div>
        <div v-else class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <div v-if="audioErrorMessage">
            <AlertMessage class="mt-2" v-bind:message="audioErrorMessage"/>
          </div>
          <div v-else-if="skyWayErrorMessage">
            <AlertMessage class="mt-2" v-bind:message="skyWayErrorMessage"/>
          </div>
          <div v-else>
            <div v-if="remoteMediaStream" class="flex flex-col items-center w-full">
              <img class="w-full md:w-3/5" src="/user-side-consultation/consultant-silhouette.png" />
              <audio v-bind:srcObject.prop="remoteMediaStream" autoplay>
                <p class="mt-4 font-bold text-xl">使われているブラウザではサービスを利用できません。他のブラウザをお使い下さい。</p>
              </audio>
              <p class="mt-2 text-xl text-center">相手側の音声が聞き取りづらいとき、一度退室し、再度入室するよう相手に促して下さい。再入室後、相手音声の高さ（低さ）が変化します。</p>
            </div>
            <div v-else>
              <h3 class="font-bold text-2xl text-center">相手が入室するまでお待ち下さい。</h3>
            </div>
          </div>
        </div>
      </div>
      <div v-if="getConsultantDetailErrMessage">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="getConsultantDetailErrMessage"/>
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
import { defineComponent, onMounted, onUnmounted, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRoute, useRouter } from 'vue-router'
import { useGetUserSideInfo } from '@/util/personalized/user-side-consultation/useGetUserSideInfo'
import { GetUserSideInfoResp } from '@/util/personalized/user-side-consultation/GetUserSideInfoResp'
import { Message } from '@/util/Message'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { useGetConsultantDetail } from '@/util/personalized/consultant-detail/useGetConsultantDetail'
import { GetConsultantDetailResp } from '@/util/personalized/consultant-detail/GetConsultantDetailResp'
import { ConsultantDetail } from '@/util/personalized/consultant-detail/ConsultantDetail'
import { convertYearsOfServiceValue, convertEmployedValue, convertContractTypeValue, convertIsManagerValue, convertIsNewGraduateValue } from '@/util/personalized/ConsultantDetailConverter'
import { UserSideInfo } from '@/util/personalized/user-side-consultation/UserSideInfo'
import { useSetupSkyWay } from '@/util/personalized/skyway/useSetUpSkyWay'
import { ProcessedAudio } from '@/util/personalized/processed-audio/ProcessedAudio'
import { ProcessedAudioError } from '@/util/personalized/processed-audio/ProcessedAudioError'
import { SkyWayAudioMeetingRoom } from '@/util/personalized/skyway/SkyWayAudioMeetingRoom'
import { SkyWayOriginatedError } from '@/util/personalized/skyway/SkyWayOriginatedError'

export default defineComponent({
  name: 'UserSideConsultationPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const route = useRoute()
    const consultationId = route.params.consultation_id as string
    const consultantId = route.params.consultant_id as string
    const router = useRouter()

    const getUserSideInfoErrMessage = ref(null as string | null)
    const {
      getUserSideInfoDone,
      getUserSideInfoFunc
    } = useGetUserSideInfo()
    const audioTestDone = ref(false)
    const userSideInfo = ref(null as UserSideInfo | null)

    const audioErrorMessage = ref(null as string | null)
    let processedAudio: ProcessedAudio | null

    const {
      skyWayErrorMessage,
      remoteMediaStream,
      setupSkyWay
    } = useSetupSkyWay()
    let audioMeetingRoom = null as SkyWayAudioMeetingRoom | null

    const getConsultantDetailErrMessage = ref(null as string | null)
    const consultantDetail = ref(null as ConsultantDetail | null)
    const {
      getConsultantDetailDone,
      getConsultantDetailFunc
    } = useGetConsultantDetail()

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

    const processGetUserSideInfo = async () => {
      await releaseAllResources()
      try {
        const resp = await getUserSideInfoFunc(consultationId, audioTestDone.value)
        if (!(resp instanceof GetUserSideInfoResp)) {
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
          getUserSideInfoErrMessage.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
        userSideInfo.value = resp.getUserSideInfo()
        if (!userSideInfo.value) {
          getUserSideInfoErrMessage.value = Message.UNEXPECTED_ERR
          return
        }
      } catch (e) {
        getUserSideInfoErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
        return
      }

      try {
        const pa = new ProcessedAudio()
        processedAudio = pa
        await pa.init()

        const amr = new SkyWayAudioMeetingRoom()
        audioMeetingRoom = amr
        await amr.init(userSideInfo.value.token, userSideInfo.value.room_name, userSideInfo.value.member_name, pa.getAudioMediaStreamTrack())
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
          getConsultantDetailErrMessage.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
        consultantDetail.value = resp.getConsultantDetail()
      } catch (e) {
        getConsultantDetailErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    onMounted(async () => {
      await processGetConsultantDetail()
    })

    onUnmounted(async () => {
      await releaseAllResources()
    })

    const leaveConsultationRoom = async () => {
      await router.push('/schedule')
    }

    return {
      getUserSideInfoErrMessage,
      getUserSideInfoDone,
      audioTestDone,
      processGetUserSideInfo,
      userSideInfo,
      audioErrorMessage,
      remoteMediaStream,
      skyWayErrorMessage,
      getConsultantDetailErrMessage,
      getConsultantDetailDone,
      consultantDetail,
      convertYearsOfServiceValue,
      convertEmployedValue,
      convertContractTypeValue,
      convertIsManagerValue,
      convertIsNewGraduateValue,
      leaveConsultationRoom
    }
  }
})
</script>
