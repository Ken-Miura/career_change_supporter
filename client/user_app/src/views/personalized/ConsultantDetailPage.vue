<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getConsultantDetailDone" class="m-6">
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
          <h3 data-test="consultant-detail-label" class="font-bold text-xl lg:text-2xl">コンサルタント詳細</h3>
          <div v-if="consultantDetail !== null" class="m-4 text-base lg:text-2xl grid grid-cols-4">
            <div data-test="consultant-id-label" class="mt-2 justify-self-start col-span-3">コンサルタントID</div><div data-test="consultant-id-value" class="mt-2 justify-self-start col-span-1">{{ consultantDetail.consultant_id }}</div>
            <div data-test="fee-per-hour-in-yen-label" class="mt-2 justify-self-start col-span-3">相談一回（１時間）の相談料</div><div data-test="fee-per-hour-in-yen-value" class="mt-2 justify-self-start col-span-1">{{ consultantDetail.fee_per_hour_in_yen }}円</div>
            <div data-test="rating-label" class="mt-2 justify-self-start col-span-3">評価（評価件数：{{ consultantDetail.num_of_rated }} 件）</div><div data-test="rating-value" class="mt-2 justify-self-start col-span-1"><span v-if="consultantDetail.rating !== null">{{ consultantDetail.rating }}</span><span v-else>0</span>/5</div>
            <div data-test="career-label" class="mt-5 justify-self-start col-span-4 font-bold text-xl lg:text-2xl">職務経歴</div>
            <div class="mt-2 justify-self-start col-span-4 flex flex-col justify-center w-full">
              <ul>
                <li v-for="(consultantCareerDetail, index) in consultantDetail.careers" v-bind:key="consultantCareerDetail.counsultant_career_detail_id">
                  <div v-bind:data-test="'career-detail-' + index" class="mt-2">
                    <div data-test="career-detail-label" class="bg-gray-600 text-white font-bold text-lg lg:text-xl rounded-t px-4 py-2">職務経歴{{ index + 1 }}</div>
                    <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-base lg:text-xl grid grid-cols-2">
                      <div data-test="company-name-label" class="mt-2 justify-self-start col-span-1">勤務先名称</div><div data-test="company-name-value" class="mt-2 justify-self-start col-span-1">{{ consultantCareerDetail.company_name }}</div>
                      <div data-test="department-name-label" v-if="consultantCareerDetail.department_name" class="mt-2 justify-self-start col-span-1">部署名</div><div data-test="department-name-value" v-if="consultantCareerDetail.department_name" class="mt-2 justify-self-start col-span-1">{{ consultantCareerDetail.department_name }}</div>
                      <div data-test="office-label" v-if="consultantCareerDetail.office" class="mt-2 justify-self-start col-span-1">勤務地</div><div data-test="office-value" v-if="consultantCareerDetail.office" class="mt-2 justify-self-start col-span-1">{{ consultantCareerDetail.office }}</div>
                      <div data-test="years-of-service-label" class="mt-2 justify-self-start col-span-1">在籍年数</div><div data-test="years-of-service-value" class="mt-2 justify-self-start col-span-1">{{ convertYearsOfServiceValue(consultantCareerDetail.years_of_service) }}</div>
                      <div data-test="employed-label" class="mt-2 justify-self-start col-span-1">在籍の有無</div><div data-test="employed-value" class="mt-2 justify-self-start col-span-1">{{ convertEmployedValue(consultantCareerDetail.employed) }}</div>
                      <div data-test="contract-type-label" class="mt-2 justify-self-start col-span-1">雇用形態</div><div data-test="contract-type-value" class="mt-2 justify-self-start col-span-1">{{ convertContractTypeValue(consultantCareerDetail.contract_type) }}</div>
                      <div data-test="profession-label" v-if="consultantCareerDetail.profession" class="mt-2 justify-self-start col-span-1">職種</div><div data-test="profession-value" v-if="consultantCareerDetail.profession" class="mt-2 justify-self-start col-span-1">{{ consultantCareerDetail.profession }}</div>
                      <div data-test="annual-income-in-man-yen-label" v-if="consultantCareerDetail.annual_income_in_man_yen" class="mt-2 justify-self-start col-span-1">年収</div><div data-test="annual-income-in-man-yen-value" v-if="consultantCareerDetail.annual_income_in_man_yen" class="mt-2 justify-self-start col-span-1">{{ consultantCareerDetail.annual_income_in_man_yen }}万円</div>
                      <div data-test="is-manager-label" class="mt-2 justify-self-start col-span-1">管理職区分</div><div data-test="is-manager-value" class="mt-2 justify-self-start col-span-1">{{ convertIsManagerValue(consultantCareerDetail.is_manager) }}</div>
                      <div data-test="position-name-label" v-if="consultantCareerDetail.position_name" class="mt-2 justify-self-start col-span-1">職位</div><div data-test="position-name-value" v-if="consultantCareerDetail.position_name" class="mt-2 justify-self-start col-span-1">{{ consultantCareerDetail.position_name }}</div>
                      <div data-test="is-new-graduate-label" class="mt-2 justify-self-start col-span-1">入社区分</div><div data-test="is-new-graduate-value" class="mt-2 justify-self-start col-span-1">{{ convertIsNewGraduateValue(consultantCareerDetail.is_new_graduate) }}</div>
                      <div data-test="note-label" v-if="consultantCareerDetail.note" class="mt-2 justify-self-start col-span-2">備考</div>
                      <div data-test="note-value" v-if="consultantCareerDetail.note" class="mt-2 ml-2 justify-self-start col-span-2 whitespace-pre-wrap">{{ consultantCareerDetail.note }}</div>
                    </div>
                  </div>
                </li>
              </ul>
            </div>
          </div>
          <p v-else class="m-4 text-xl">コンサルタントの詳細を取得出来ませんでした。</p>
          <button data-test="move-to-request-consultantion-page-button" v-on:click="moveToRequestConsultationPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">相談を申し込む</button>
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
import { Message } from '@/util/Message'
import { useGetConsultantDetail } from '@/util/personalized/consultant-detail/useGetConsultantDetail'
import { GetConsultantDetailResp } from '@/util/personalized/consultant-detail/GetConsultantDetailResp'
import { ConsultantDetail } from '@/util/personalized/consultant-detail/ConsultantDetail'
import { convertYearsOfServiceValue, convertEmployedValue, convertContractTypeValue, convertIsManagerValue, convertIsNewGraduateValue } from '@/util/personalized/ConsultantDetailConverter'

export default defineComponent({
  name: 'ConsultantDetailPage',
  components: {
    TheHeader,
    WaitingCircle,
    AlertMessage
  },
  setup () {
    const error = reactive({
      exists: false,
      message: ''
    })
    const consultantDetail = ref(null as ConsultantDetail | null)
    const router = useRouter()
    const route = useRoute()
    const {
      getConsultantDetailDone,
      getConsultantDetailFunc
    } = useGetConsultantDetail()
    const consultantId = route.params.consultant_id as string

    onMounted(async () => {
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
    })

    const moveToRequestConsultationPage = async () => {
      await router.push({ name: 'RequestConsultationPage', params: { consultant_id: consultantId } })
    }

    return {
      error,
      consultantDetail,
      getConsultantDetailDone,
      getConsultantDetailFunc,
      convertYearsOfServiceValue,
      convertEmployedValue,
      convertContractTypeValue,
      convertIsManagerValue,
      convertIsNewGraduateValue,
      moveToRequestConsultationPage
    }
  }
})
</script>
