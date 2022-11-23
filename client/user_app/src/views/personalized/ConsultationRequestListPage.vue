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
          <h3 class="font-bold text-2xl">相談申し込み一覧</h3>
          <p class="mt-2 text-lg">詳細を押し、相談申し込みの内容を確認して下さい。相談申し込みは、最大で{{ MAX_NUM_OF_CONSULTATION_REQUESTS }}件表示されます。</p>
          <div class="mt-4 ml-4">
            <div v-if="consultationRequests.length === 0">
              Empty
            </div>
            <div v-else>
              <ul>
                <li v-for="(consultationReq, index) in consultationRequests" v-bind:key="consultationReq.consultation_req_id">
                  consultationReq: {{ consultationReq }}, index: {{ index }}
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
    return {
      getConsultationRequestsDone,
      error,
      consultationRequests,
      MAX_NUM_OF_CONSULTATION_REQUESTS
    }
  }
})
</script>
