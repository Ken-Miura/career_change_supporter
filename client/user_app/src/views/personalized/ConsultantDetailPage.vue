<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getConsultantDetailDone" class="m-6">
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
          <h3 class="font-bold text-2xl">コンサルタント詳細</h3>
          <div v-if="consultantDetail !== null" class="m-4 text-2xl grid grid-cols-3">
            <div class="mt-2 justify-self-start col-span-2">コンサルタントID</div><div class="mt-2 justify-self-start col-span-1">{{ consultantDetail.consultant_id }}</div>
            <div class="mt-2 justify-self-start col-span-2">相談一回（１時間）の相談料</div><div class="mt-2 justify-self-start col-span-1">{{ consultantDetail.fee_per_hour_in_yen }}円</div>
            <div class="mt-2 justify-self-start col-span-2">評価（評価件数：{{ consultantDetail.num_of_rated }} 件）</div><div class="mt-2 justify-self-start col-span-1"><span v-if="consultantDetail.rating !== null">{{ consultantDetail.rating }}</span><span v-else>0</span>/5</div>
            <div class="mt-5 justify-self-start col-span-3 font-bold text-2xl">職務経歴</div>
            <div class="mt-2 justify-self-start col-span-3 flex flex-col justify-center w-full">
              <ul>
                <li v-for="(consultantCareerDetail, index) in consultantDetail.careers" v-bind:key="consultantCareerDetail.counsultant_career_detail_id">
                  <div class="mt-2">
                    <div class="bg-gray-600 text-white font-bold text-xl rounded-t px-4 py-2">職務経歴{{ index + 1 }}</div>
                    <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                      <div class="mt-2 justify-self-start col-span-1">勤務先名称</div><div class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.company_name }}</div>
                      <div v-if="consultantCareerDetail.department_name" class="mt-2 justify-self-start col-span-1">部署名</div><div v-if="consultantCareerDetail.department_name" class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.department_name }}</div>
                      <div v-if="consultantCareerDetail.office" class="mt-2 justify-self-start col-span-1">勤務地</div><div v-if="consultantCareerDetail.office" class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.office }}</div>
                      <div class="mt-2 justify-self-start col-span-1">在籍年数</div><div class="mt-2 justify-self-start col-span-2">{{ convertYearsOfServiceValue(consultantCareerDetail.years_of_service) }}</div>
                      <div class="mt-2 justify-self-start col-span-1">在籍の有無</div><div class="mt-2 justify-self-start col-span-2">{{ convertEmployedValue(consultantCareerDetail.employed) }}</div>
                      <div class="mt-2 justify-self-start col-span-1">雇用形態</div><div class="mt-2 justify-self-start col-span-2">{{ convertContractTypeValue(consultantCareerDetail.contract_type) }}</div>
                      <div v-if="consultantCareerDetail.profession" class="mt-2 justify-self-start col-span-1">職種</div><div v-if="consultantCareerDetail.profession" class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.profession }}</div>
                      <div v-if="consultantCareerDetail.annual_income_in_man_yen" class="mt-2 justify-self-start col-span-1">年収</div><div v-if="consultantCareerDetail.annual_income_in_man_yen" class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.annual_income_in_man_yen }}万円</div>
                      <div class="mt-2 justify-self-start col-span-1">管理職区分</div><div class="mt-2 justify-self-start col-span-2">{{ convertIsManagerValue(consultantCareerDetail.is_manager) }}</div>
                      <div v-if="consultantCareerDetail.position_name" class="mt-2 justify-self-start col-span-1">職位</div><div v-if="consultantCareerDetail.position_name" class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.position_name }}</div>
                      <div class="mt-2 justify-self-start col-span-1">入社区分</div><div class="mt-2 justify-self-start col-span-2">{{ convertIsNewGraduateValue(consultantCareerDetail.is_new_graduate) }}</div>
                      <div v-if="consultantCareerDetail.note" class="mt-2 justify-self-start col-span-1">備考</div><div v-if="consultantCareerDetail.note" class="mt-2 justify-self-start col-span-2">{{ consultantCareerDetail.note }}</div>
                    </div>
                  </div>
                </li>
              </ul>
            </div>
          </div>
          <p v-else class="m-4 text-xl">コンサルタントの詳細を取得出来ませんでした。</p>
          <button class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">相談を申し込む</button>
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

    const convertYearsOfServiceValue = (yearsOfService: string): string => {
      if (yearsOfService === 'LESS_THAN_THREE_YEARS') {
        return '3年未満'
      } else if (yearsOfService === 'THREE_YEARS_OR_MORE_LESS_THAN_FIVE_YEARS') {
        return '3年以上5年未満'
      } else if (yearsOfService === 'FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS') {
        return '5年以上10年未満'
      } else if (yearsOfService === 'TEN_YEARS_OR_MORE_LESS_THAN_FIFTEEN_YEARS') {
        return '10年以上15年未満'
      } else if (yearsOfService === 'FIFTEEN_YEARS_OR_MORE_LESS_THAN_TWENTY_YEARS') {
        return '15年以上20年未満'
      } else if (yearsOfService === 'TWENTY_YEARS_OR_MORE') {
        return '20年以上'
      } else {
        return '不明'
      }
    }

    const convertEmployedValue = (employed: boolean): string => {
      if (employed) {
        return '在籍中'
      } else {
        return '退職済'
      }
    }

    const convertContractTypeValue = (contractType: string): string => {
      if (contractType === 'regular') {
        return '正社員'
      } else if (contractType === 'contract') {
        return '契約社員'
      } else if (contractType === 'other') {
        return 'その他'
      } else {
        return '不明'
      }
    }

    const convertIsManagerValue = (isManager: boolean): string => {
      if (isManager) {
        return '管理職'
      } else {
        return '非管理職'
      }
    }

    const convertIsNewGraduateValue = (isNewGraduate: boolean): string => {
      if (isNewGraduate) {
        return '新卒入社'
      } else {
        return '中途入社'
      }
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
      convertIsNewGraduateValue
    }
  }
})
</script>
