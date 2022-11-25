<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getConsultationRequestsDone" class="m-6">
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
          <h3 data-test="consultation-request-list-label" class="font-bold text-2xl">相談申し込み一覧</h3>
          <p data-test="consultation-request-list-description" class="mt-2 text-lg">相談申し込みの内容を確認し、申し込みの了承または拒否をして下さい。相談申し込みは、最大で{{ MAX_NUM_OF_CONSULTATION_REQUESTS }}件表示されます。</p>
          <div class="mt-4 ml-4">
            <div v-if="consultationRequests.length === 0">
              <p data-test="no-consultation-request-found" class="mt-2 text-lg">相談申し込みはありません。</p>
            </div>
            <div v-else>
              <ul>
                <li v-for="consultationReq in consultationRequests" v-bind:key="consultationReq.consultation_req_id">
                  <div v-bind:data-test="'consultation-req-id-' + consultationReq.consultation_req_id" class="mt-4">
                    <div data-test="consultation-req-id" class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">相談申し込み番号: {{ consultationReq.consultation_req_id }}</div>
                    <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                      <div data-test="user-id" class="mt-4 justify-self-start col-span-2">ユーザーID（{{ consultationReq.user_account_id }}）からの相談申し込み</div>
                      <button data-test="move-to-consultation-req-detail-page-btn" v-on:click="moveToConsultationRequestDetailPage(consultationReq.consultation_req_id)" class="mt-2 col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">詳細を確認する</button>
                    </div>
                  </div>
                </li>
              </ul>
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
import { useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { useGetConsultationRequests } from '@/util/personalized/consultation-request-list/useGetConsultationRequests'
import { Message } from '@/util/Message'
import { GetConsultationRequestsResp } from '@/util/personalized/consultation-request-list/GetConsultationRequestsResp'
import { ConsultationRequestDescription } from '@/util/personalized/consultation-request-list/ConsultationRequestsResult'
import { MAX_NUM_OF_CONSULTATION_REQUESTS } from '@/util/personalized/consultation-request-list/MaxNumOfConsultationRequests'

export default defineComponent({
  name: 'ConsultationRequestListPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const {
      getConsultationRequestsDone,
      getConsultationRequestsFunc
    } = useGetConsultationRequests()
    const error = reactive({
      exists: false,
      message: ''
    })
    const consultationRequests = ref([] as ConsultationRequestDescription[])
    const router = useRouter()

    onMounted(async () => {
      try {
        const resp = await getConsultationRequestsFunc()
        if (!(resp instanceof GetConsultationRequestsResp)) {
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
        consultationRequests.value = resp.getConsultationRequestsResult().consultation_requests
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const moveToConsultationRequestDetailPage = async (consultationReqId: number) => {
      await router.push({ name: 'ConsultationRequestDetailPage', params: { consultation_req_id: consultationReqId } })
    }

    return {
      getConsultationRequestsDone,
      error,
      consultationRequests,
      MAX_NUM_OF_CONSULTATION_REQUESTS,
      moveToConsultationRequestDetailPage
    }
  }
})
</script>
