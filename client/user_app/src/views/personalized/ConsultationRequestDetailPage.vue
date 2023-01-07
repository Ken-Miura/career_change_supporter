<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!(getConsultationRequestDetailDone && postConsultationRequestRejectionDone && postConsultationRequestAcceptanceDone)" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="error.exists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="error.message"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <div class="text-xl text-center" v-if="consultationReqDetail === null">
            {{ unexpectedErrMsg }}
          </div>
          <div v-else>
            <h3 data-test="consultation-req-detail-label" class="font-bold text-2xl">相談申し込み詳細</h3>
            <p class="mt-2 text-lg">詳細を確認し、相談申し込みを受けるかどうか選択して下さい。</p>
            <div class="grid grid-cols-2 mt-4 ml-4">
              <h3 class="text-xl justify-self-start col-span-1">ユーザーID</h3><h3 class="text-xl justify-self-start col-span-1">{{ consultationReqDetail.user_account_id }}</h3>
              <div class="mt-3 justify-self-start col-span-1 text-xl">評価</div><div class="mt-3 justify-self-start col-span-1 text-xl"><span v-if="consultationReqDetail.user_rating !== null">{{ consultationReqDetail.user_rating }}</span><span v-else>0</span>/5（評価件数：{{ consultationReqDetail.num_of_rated_of_user }} 件）</div>
              <p class="mt-3 justify-self-start col-span-1 text-xl">相談料</p><p class="mt-3 justify-self-start col-span-1 text-xl">{{ consultationReqDetail.fee_per_hour_in_yen }} 円</p>
            </div>
            <div class="flex flex-col justify-center mt-6 ml-4">
              <p class="font-bold text-xl">希望相談開始日時候補一覧</p>
              <p class="mt-2 ml-2 text-xl">下記の候補一覧の内、一つを選択して下さい。相談は開始日時から1時間です。</p>
              <div class="mt-4 ml-4">
                <div class="flex items-center mb-4">
                  <input v-model="picked" type="radio" value="1" name="candidates" class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600">
                  <label class="ml-2 text-xl font-medium text-gray-900 dark:text-gray-300">第一希望: {{ consultationReqDetail.first_candidate_in_jst.year }}年{{ consultationReqDetail.first_candidate_in_jst.month }}月{{ consultationReqDetail.first_candidate_in_jst.day }}日{{ consultationReqDetail.first_candidate_in_jst.hour }}時</label>
                </div>
                <div class="flex items-center mb-4">
                  <input v-model="picked" type="radio" value="2" name="candidates" class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600">
                  <label class="ml-2 text-xl font-medium text-gray-900 dark:text-gray-300">第二希望: {{ consultationReqDetail.second_candidate_in_jst.year }}年{{ consultationReqDetail.second_candidate_in_jst.month }}月{{ consultationReqDetail.second_candidate_in_jst.day }}日{{ consultationReqDetail.second_candidate_in_jst.hour }}時</label>
                </div>
                <div class="flex items-center mb-4">
                  <input v-model="picked" type="radio" value="3" name="candidates" class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600">
                  <label class="ml-2 text-xl font-medium text-gray-900 dark:text-gray-300">第三希望: {{ consultationReqDetail.third_candidate_in_jst.year }}年{{ consultationReqDetail.third_candidate_in_jst.month }}月{{ consultationReqDetail.third_candidate_in_jst.day }}日{{ consultationReqDetail.third_candidate_in_jst.hour }}時</label>
                </div>
              </div>
            </div>
            <div class="mt-4 ml-4 text-2xl justify-self-start col-span-6 pt-3">
              <p>確認事項</p>
              <p class="text-lg">相談申し込みを受け付けるためには、下記に記載の内容が正しいことを確認し、チェックをつけて下さい</p>
            </div>
            <div class="mt-2 ml-4 justify-self-start col-span-6 py-1 rounded bg-gray-200">
              <div class="m-4 text-xl grid grid-cols-6 justify-center items-center">
                <div class="col-span-5">
                  <ul class="ml-4 space-y-2 list-disc">
                    <li>私は社外秘とは何かを理解しており、それを口外することはありません。</li>
                    <li>私は相談申し込みを受けた後、それをキャンセルできないことを理解しています。</li>
                  </ul>
                </div>
                <input v-model="userChecked" type="checkbox" class="ml-5 col-span-1 justify-self-center bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
              </div>
            </div>
            <div class="flex justify-center mt-8">
              <button v-on:click="rejectConsultationReq" class="mr-10 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">相談申し込みを拒否する</button>
              <button v-on:click="takeConsultationReq" v-bind:disabled="!userChecked" class="ml-10 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">相談申し込みを受ける</button>
            </div>
            <div v-if="errorBelowBtn.exists">
              <AlertMessage class="mt-6" v-bind:message="errorBelowBtn.message"/>
            </div>
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
import { useRoute, useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { useGetConsultationRequestDetail } from '@/util/personalized/consultation-request-detail/useGetConsultationRequestDetail'
import { usePostConsultationRequestRejection } from '@/util/personalized/consultation-request-detail/usePostConsultationRequestRejection'
import { usePostConsultationRequestAcceptance } from '@/util/personalized/consultation-request-detail/usePostConsultationRequestAcceptance'
import { Message } from '@/util/Message'
import { GetConsultationRequestDetailResp } from '@/util/personalized/consultation-request-detail/GetConsultationRequestDetailResp'
import { ConsultationRequestDetail } from '@/util/personalized/consultation-request-detail/ConsultationRequestDetail'
import { ConsultationRequestRejectionParam } from '@/util/personalized/consultation-request-detail/ConsultationRequestRejectionParam'
import { PostConsultationRequestRejectionResp } from '@/util/personalized/consultation-request-detail/PostConsultationRequestRejectionResp'
import { PostConsultationRequestAcceptanceResp } from '@/util/personalized/consultation-request-detail/PostConsultationRequestAcceptanceResp'
import { ConsultationRequestAcceptanceParam } from '@/util/personalized/consultation-request-detail/ConsultationRequestAcceptanceParam'

export default defineComponent({
  name: 'ConsultationRequestDetailPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const {
      getConsultationRequestDetailDone,
      getConsultationRequestDetailFunc
    } = useGetConsultationRequestDetail()
    const error = reactive({
      exists: false,
      message: ''
    })
    const router = useRouter()
    const route = useRoute()
    const consultationReqId = route.params.consultation_req_id as string
    const consultationReqDetail = ref(null as ConsultationRequestDetail | null)
    const picked = ref('')
    const unexpectedErrMsg = Message.UNEXPECTED_ERR
    const userChecked = ref(false)
    const {
      postConsultationRequestRejectionDone,
      postConsultationRequestRejectionFunc
    } = usePostConsultationRequestRejection()
    const {
      postConsultationRequestAcceptanceDone,
      postConsultationRequestAcceptanceFunc
    } = usePostConsultationRequestAcceptance()
    const errorBelowBtn = reactive({
      exists: false,
      message: ''
    })

    onMounted(async () => {
      try {
        const resp = await getConsultationRequestDetailFunc(consultationReqId)
        if (!(resp instanceof GetConsultationRequestDetailResp)) {
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
        consultationReqDetail.value = resp.getConsultationRequestDetail()
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const rejectConsultationReq = async () => {
      try {
        const param = { consultation_req_id: parseInt(consultationReqId) } as ConsultationRequestRejectionParam
        const resp = await postConsultationRequestRejectionFunc(param)
        if (!(resp instanceof PostConsultationRequestRejectionResp)) {
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
          errorBelowBtn.exists = true
          errorBelowBtn.message = createErrorMessage(resp.getApiError().getCode())
          return
        }
        await router.push('/consultation-request-rejection')
      } catch (e) {
        errorBelowBtn.exists = true
        errorBelowBtn.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const checkIfPickedValueIsInValidRange = (pickedValue: string): boolean => {
      if (pickedValue === '1' || pickedValue === '2' || pickedValue === '3') {
        return true
      }
      return false
    }

    const takeConsultationReq = async () => {
      try {
        const reqId = parseInt(consultationReqId)
        if (!(reqId > 0)) {
          errorBelowBtn.exists = true
          errorBelowBtn.message = Message.NON_POSITIVE_CONSULTATION_REQ_ID_MESSAGE
          return
        }
        if (!checkIfPickedValueIsInValidRange(picked.value)) {
          errorBelowBtn.exists = true
          errorBelowBtn.message = Message.INVALID_CANDIDATE_MESSAGE
          return
        }
        if (!userChecked.value) {
          errorBelowBtn.exists = true
          errorBelowBtn.message = Message.USER_DOES_NOT_CHECK_CONFIRMATION_ITEMS_MESSAGE
          return
        }
        const param = {
          consultation_req_id: reqId,
          picked_candidate: parseInt(picked.value),
          user_checked: userChecked.value
        } as ConsultationRequestAcceptanceParam
        const resp = await postConsultationRequestAcceptanceFunc(param)
        if (!(resp instanceof PostConsultationRequestAcceptanceResp)) {
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
          errorBelowBtn.exists = true
          errorBelowBtn.message = createErrorMessage(resp.getApiError().getCode())
          return
        }
        await router.push('/consultation-request-acceptance')
      } catch (e) {
        errorBelowBtn.exists = true
        errorBelowBtn.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    return {
      getConsultationRequestDetailDone,
      postConsultationRequestRejectionDone,
      postConsultationRequestAcceptanceDone,
      error,
      consultationReqDetail,
      picked,
      unexpectedErrMsg,
      userChecked,
      rejectConsultationReq,
      takeConsultationReq,
      errorBelowBtn
    }
  }
})
</script>
