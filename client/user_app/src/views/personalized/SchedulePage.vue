<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getConsultationsDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="error.exists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="error.message"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
          <h3 data-test="user-side-consultation-label" class="font-bold text-xl lg:text-2xl">あなたが申し込んだ相談</h3>
          <div v-if="consultationsResult.user_side_consultations.length !== 0" class="m-2 text-base lg:text-xl">
            <ul>
              <li v-for="user_side_consultation in consultationsResult.user_side_consultations" v-bind:key="user_side_consultation.consultation_id">
                <div v-bind:data-test="'user-side-consultation-id-' + user_side_consultation.consultation_id" class="mt-4">
                  <div data-test="consultant-id-label" class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">コンサルタントID（{{ user_side_consultation.consultant_id }}）への相談</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black flex flex-col lg:flex-row justify-between">
                    <div data-test="user-side-consultation-date-time" class="mt-0 lg:mt-4">相談開始日時：{{ user_side_consultation.meeting_date_time_in_jst.year }}年{{ user_side_consultation.meeting_date_time_in_jst.month }}月{{ user_side_consultation.meeting_date_time_in_jst.day }}日{{ user_side_consultation.meeting_date_time_in_jst.hour }}時</div>
                    <button data-test="move-to-user-side-consultation-page" v-on:click="moveToUserSideConsultationPage(user_side_consultation.consultation_id, user_side_consultation.consultant_id)" class="mt-2 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">相談室へ入室する</button>
                  </div>
                </div>
              </li>
            </ul>
          </div>
          <div v-else class="m-6">
            <p data-test="no-user-side-consultation-label" class="text-base lg:text-xl">あなたが申し込んだ相談はありません</p>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
          <h3 data-test="consultant-side-consultation-label" class="font-bold text-xl lg:text-2xl">あなたが受け付けた相談</h3>
          <div v-if="consultationsResult.consultant_side_consultations.length !== 0" class="m-2 text-base lg:text-xl">
            <ul>
              <li v-for="consultant_side_consultation in consultationsResult.consultant_side_consultations" v-bind:key="consultant_side_consultation.consultation_id">
                <div v-bind:data-test="'consultant-side-consultation-id-' + consultant_side_consultation.consultation_id" class="mt-4">
                  <div data-test="user-account-id-label" class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">ユーザーID（{{ consultant_side_consultation.user_account_id }}）からの相談</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black flex flex-col lg:flex-row justify-between">
                    <div data-test="consultant-side-consultation-date-time" class="mt-0 lg:mt-4">相談開始日時：{{ consultant_side_consultation.meeting_date_time_in_jst.year }}年{{ consultant_side_consultation.meeting_date_time_in_jst.month }}月{{ consultant_side_consultation.meeting_date_time_in_jst.day }}日{{ consultant_side_consultation.meeting_date_time_in_jst.hour }}時</div>
                    <button data-test="move-to-consultant-side-consultation-page" v-on:click="moveToConsultantSideConsultationPage(consultant_side_consultation.consultation_id, consultant_side_consultation.user_account_id)" class="mt-2 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">相談室へ入室する</button>
                  </div>
                </div>
              </li>
            </ul>
          </div>
          <div v-else class="m-6">
            <p data-test="no-consultant-side-consultation-label" class="text-base lg:text-xl">あなたが受け付けた相談はありません</p>
          </div>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { Message } from '@/util/Message'
import { useGetConsultations } from '@/util/personalized/schedule/useGetConsultations'
import { GetConsultationsResp } from '@/util/personalized/schedule/GetConsultationsResp'
import { ConsultationsResult } from '@/util/personalized/schedule/ConsultationsResult'

export default defineComponent({
  name: 'SchedulePage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const {
      getConsultationsDone,
      getConsultationsFunc
    } = useGetConsultations()
    const consultationsResult = ref({ user_side_consultations: [], consultant_side_consultations: [] } as ConsultationsResult)
    const error = reactive({
      exists: false,
      message: ''
    })
    const router = useRouter()

    onMounted(async () => {
      try {
        const resp = await getConsultationsFunc()
        if (!(resp instanceof GetConsultationsResp)) {
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
        consultationsResult.value = resp.getConsultationsResult()
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const moveToUserSideConsultationPage = async (consultationId: number, consultantId: number) => {
      await router.push(`/user-side-consultation/${consultationId}/consultant/${consultantId}`)
    }

    const moveToConsultantSideConsultationPage = async (consultationId: number, userAccountId: number) => {
      await router.push(`/consultant-side-consultation/${consultationId}/user/${userAccountId}`)
    }

    return {
      getConsultationsDone,
      consultationsResult,
      error,
      moveToUserSideConsultationPage,
      moveToConsultantSideConsultationPage
    }
  }
})
</script>
