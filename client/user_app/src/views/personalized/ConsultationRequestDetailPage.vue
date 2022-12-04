<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getConsultationRequestDetailDone" class="m-6">
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
            <h3 class="font-bold text-2xl">相談申し込み詳細</h3>
            <p class="mt-2 text-lg">詳細を確認し、相談申し込みを受けるかどうか選択して下さい。</p>
            <div class="grid grid-cols-2 mt-4 ml-4">
              <h3 class="text-xl justify-self-start col-span-1">ユーザーID</h3><h3 class="text-xl justify-self-start col-span-1">2</h3>
              <div class="mt-3 justify-self-start col-span-1 text-xl">評価</div><div class="mt-3 justify-self-start col-span-1 text-xl"><span v-if="true"> 4.5</span><span v-else>0</span>/5（評価件数：21 件）</div>
              <p class="mt-3 justify-self-start col-span-1 text-xl">支払われる相談料</p><p class="mt-3 justify-self-start col-span-1 text-xl">5000 円</p>
            </div>
            <div class="flex flex-col justify-center mt-6 ml-4">
              <p class="font-bold text-xl">希望相談開始日時候補一覧</p>
              <p class="mt-2 ml-2 text-xl">下記の候補一覧の内、一つを選択して下さい。相談は開始日時から1時間です。</p>
              <div class="mt-4 ml-4">
                <div class="flex items-center mb-4">
                  <input type="radio" value="" name="default-radio" class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600">
                  <label class="ml-2 text-xl font-medium text-gray-900 dark:text-gray-300">第一希望: 2022年12月10日19時</label>
                </div>
                <div class="flex items-center mb-4">
                  <input type="radio" value="" name="default-radio" class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600">
                  <label class="ml-2 text-xl font-medium text-gray-900 dark:text-gray-300">第二希望: 2022年12月11日7時</label>
                </div>
                <div class="flex items-center mb-4">
                  <input type="radio" value="" name="default-radio" class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600">
                  <label class="ml-2 text-xl font-medium text-gray-900 dark:text-gray-300">第三希望: 2022年12月12日23時</label>
                </div>
              </div>
            </div>
            <div class="mt-4 ml-4 text-2xl justify-self-start col-span-6 pt-3">
              <p>確認事項</p>
              <p class="text-lg">相談申し込みを受け付けるためには、下記に記載の内容が正しいことを確認し、チェックをつけて下さい</p>
            </div>
            <div class="mt-2 ml-4 justify-self-start col-span-6 py-1 rounded bg-gray-200">
              <div class="m-4 text-xl grid grid-cols-6 justify-center items-center">
                <div class="col-span-5">私は社外秘が何かを理解しおり、それを口外することはありません。</div>
                <input type="checkbox" class="ml-5 col-span-1 justify-self-center bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
              </div>
            </div>
            <div class="flex justify-center mt-8">
              <button class="mr-10 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">相談申し込みを拒否する</button>
              <button class="ml-10 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">相談申し込みを受ける</button>
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
import { Message } from '@/util/Message'
import { GetConsultationRequestDetailResp } from '@/util/personalized/consultation-request-detail/GetConsultationRequestDetailResp'
import { ConsultationRequestDetail } from '@/util/personalized/consultation-request-detail/ConsultationRequestDetail'

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
    const unexpectedErrMsg = Message.UNEXPECTED_ERR
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
    return {
      getConsultationRequestDetailDone,
      error,
      consultationReqDetail,
      unexpectedErrMsg,
      errorBelowBtn
    }
  }
})
</script>
