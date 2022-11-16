<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getFeePerHourInYenForApplicationDone || !requestConsultationDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="error.exists">
        <div data-test="outer-alert-message" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="error.message"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <p data-test="description" class="text-2xl">相談開始日時に関して、第一希望、第二希望、第三希望を入力して下さい。申し込み可能な相談開始日時は、申し込み日時から{{ minDurationInDays*24 }}時間（{{ minDurationInDays }}日）以降、{{ maxDurationInDays*24 }}時間（{{ maxDurationInDays }}日）以前までとなります。</p>
          <h3 data-test="first-candidate-lablel" class="mt-4 font-bold text-2xl">相談開始日時（第一希望）</h3>
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="candidates.firstCandidateYearInJst" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in yearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div data-test="first-candidate-year-lablel" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              年
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="candidates.firstCandidateMonthInJst" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in monthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div data-test="first-candidate-month-lablel" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              月
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="candidates.firstCandidateDayInJst" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in dayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div data-test="first-candidate-day-lablel" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              日
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="candidates.firstCandidateHourInJst" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="hour in hourList" v-bind:key="hour" v-bind:value="hour">{{ hour }}</option>
              </select>
            </div>
            <div data-test="first-candidate-hour-lablel" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              時
            </div>
          </div>
          <h3 data-test="second-candidate-lablel" class="mt-4 font-bold text-2xl">相談開始日時（第二希望）</h3>
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="candidates.secondCandidateYearInJst" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in yearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div data-test="second-candidate-year-lablel" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              年
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="candidates.secondCandidateMonthInJst" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in monthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div data-test="second-candidate-month-lablel" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              月
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="candidates.secondCandidateDayInJst" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in dayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div data-test="second-candidate-day-lablel" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              日
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="candidates.secondCandidateHourInJst" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="hour in hourList" v-bind:key="hour" v-bind:value="hour">{{ hour }}</option>
              </select>
            </div>
            <div data-test="second-candidate-hour-lablel" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              時
            </div>
          </div>
          <h3 data-test="third-candidate-lablel" class="mt-4 font-bold text-2xl">相談開始日時（第三希望）</h3>
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="candidates.thirdCandidateYearInJst" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in yearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div data-test="third-candidate-year-lablel" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              年
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="candidates.thirdCandidateMonthInJst" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in monthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div data-test="third-candidate-month-lablel" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              月
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="candidates.thirdCandidateDayInJst" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in dayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div data-test="third-candidate-day-lablel" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              日
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="candidates.thirdCandidateHourInJst" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="hour in hourList" v-bind:key="hour" v-bind:value="hour">{{ hour }}</option>
              </select>
            </div>
            <div data-test="third-candidate-hour-lablel" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              時
            </div>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 data-test="consultation-detail" class="font-bold text-2xl">相談申し込み詳細</h3>
          <div class="m-4 text-2xl grid grid-cols-3">
            <div data-test="consultant-id" class="mt-2 justify-self-start col-span-2">コンサルタントID</div><div data-test="consultant-id-value" class="mt-2 justify-self-start col-span-1">{{ consultantId }}</div>
            <div data-test="fee-per-hour-in-yen" class="mt-2 justify-self-start col-span-2">相談一回（１時間）の相談料</div><div data-test="fee-per-hour-in-yen-value" class="mt-2 justify-self-start col-span-1">{{ feePerHourInYen }}円</div>
          </div>
          <h3 data-test="card-label" class="mt-4 font-bold text-2xl">クレジットカード</h3>
          <div data-test="card-area" class="m-4 text-2xl flex flex-col">
            <div class="mt-2 w-5/6" id="payjp-card-area"></div>
          </div>
          <h3 data-test="notice" class="mt-6 ml-2 text-red-500 text-xl">相談申し込み後にキャンセルや相談開始日時変更は出来ませんので、申し込み内容についてよくご確認の上、相談をお申し込み下さい。</h3>
          <button data-test="apply-for-consultation-btn" v-bind:disabled="disabled" class="mt-8 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none" v-on:click="requestConsultation">相談を申し込む</button>
          <div data-test="inner-alert-message" v-if="errorBelowBtn.exists">
            <AlertMessage class="mt-4" v-bind:message="errorBelowBtn.message"/>
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
import { defineComponent, onMounted, onUnmounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useStore } from 'vuex'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { useGetFeePerHourInYenForApplication } from '@/util/personalized/request-consultation/useGetFeePerHourInYenForApplication'
import { GetFeePerHourInYenForApplicationResp } from '@/util/personalized/request-consultation/GetFeePerHourInYenForApplicationResp'
import { Message } from '@/util/Message'
import { SET_PAY_JP } from '@/store/mutationTypes'
import { createPayJp } from '@/util/PayJp'
import { createDayList } from '@/util/personalized/request-consultation/DayList'
import { createHourList } from '@/util/personalized/request-consultation/HourList'
import { createMonthList, getCurrentMonth } from '@/util/personalized/request-consultation/MonthList'
import { createYearList, getCurrentYear } from '@/util/personalized/request-consultation/YearList'
import { getMinDurationBeforeConsultationInDays, getMaxDurationBeforeConsultationInDays } from '@/util/personalized/request-consultation/DurationBeforeConsultation'
import { convertRemToPx } from '@/util/personalized/request-consultation/FontSizeConverter'
import { FinishRequestConsultation } from '@/util/personalized/request-consultation/FinishRequestConsultation'
import { PostFinishRequestConsultationResp } from '@/util/personalized/request-consultation/PostFinishRequestConsultationResp'
import { useRequestConsultationDone } from '@/util/personalized/request-consultation/useRequestConsultationDone'
import { postFinishRequestConsultation } from '@/util/personalized/request-consultation/PostFinishRequestConsultation'
import { useCandidate } from '@/util/personalized/request-consultation/useCandidate'
import { ConsultationRequest } from '@/util/personalized/request-consultation/ConsultationRequest'
import { postRequestConsultation } from '@/util/personalized/request-consultation/PostRequestConsultation'
import { PostRequestConsultationResp } from '@/util/personalized/request-consultation/PostRequestConsultationResp'
import { checkIfCandidateIsInValidRange } from '@/util/personalized/request-consultation/CheckIfCandidateIsInValidRange'

export default defineComponent({
  name: 'RequestConsultationPage',
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
    const router = useRouter()
    const route = useRoute()
    const store = useStore()
    const consultantId = route.params.consultant_id as string
    const currentMonth = getCurrentMonth()
    const monthList = ref(createMonthList(currentMonth))
    const currentYear = getCurrentYear()
    const yearList = ref(createYearList(currentMonth, currentYear))
    const dayList = ref(createDayList())
    const hourList = ref(createHourList())
    const minDurationInDays = getMinDurationBeforeConsultationInDays()
    const maxDurationInDays = getMaxDurationBeforeConsultationInDays()
    const { candidates, allCandidatesAreNotEmpty, sameCandidatesExist } = useCandidate()
    // PAY.JPから型定義が提供されていないため、anyでの扱いを許容する
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let cardElement = null as any
    const {
      getFeePerHourInYenForApplicationDone,
      getFeePerHourInYenForApplicationFunc
    } = useGetFeePerHourInYenForApplication()
    const feePerHourInYen = ref(0 as number)
    const {
      requestConsultationDone,
      startRequestConsultation,
      finishRequestConsultation,
      disabled,
      disableBtn,
      enableBtn
    } = useRequestConsultationDone()
    const errorBelowBtn = reactive({
      exists: false,
      message: ''
    })

    onMounted(async () => {
      try {
        const resp = await getFeePerHourInYenForApplicationFunc(consultantId)
        if (!(resp instanceof GetFeePerHourInYenForApplicationResp)) {
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
        feePerHourInYen.value = resp.getFeePerHourInYenForApplication()

        let payjp = store.state.payJp
        if (payjp === null) {
          payjp = await createPayJp()
          store.commit(SET_PAY_JP, payjp)
        }
        const elements = await payjp.elements()
        if (elements === null) {
          error.exists = true
          error.message = `${Message.UNEXPECTED_ERR}: elements is null`
          return
        }
        const px = convertRemToPx(1.5)
        cardElement = elements.create('card', {
          style: {
            base: {
              color: 'black',
              fontSize: px.toString() + 'px'
            },
            invalid: {
              color: 'red'
            }
          }
        })
        if (cardElement === null) {
          error.exists = true
          error.message = `${Message.UNEXPECTED_ERR}: cardElement is null`
          return
        }
        cardElement.mount('#payjp-card-area')
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    onUnmounted(async () => {
      cardElement.unmount()
      cardElement = null
    })

    const requestConsultation = async () => {
      try {
        disableBtn()

        const payjp = store.state.payJp
        if (payjp === null) {
          errorBelowBtn.exists = true
          errorBelowBtn.message = `${Message.UNEXPECTED_ERR}: payjp is null`
          return
        }

        if (!allCandidatesAreNotEmpty.value) {
          errorBelowBtn.exists = true
          errorBelowBtn.message = `${Message.NOT_ALL_CANDIDATES_ARE_INPUT_MESSAGE}`
          return
        }
        if (sameCandidatesExist.value) {
          errorBelowBtn.exists = true
          errorBelowBtn.message = `${Message.DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE}`
          return
        }
        if (!checkIfCandidateIsInValidRange(candidates.firstCandidateYearInJst, candidates.firstCandidateMonthInJst, candidates.firstCandidateDayInJst, candidates.firstCandidateHourInJst) ||
          !checkIfCandidateIsInValidRange(candidates.secondCandidateYearInJst, candidates.secondCandidateMonthInJst, candidates.secondCandidateDayInJst, candidates.secondCandidateHourInJst) ||
          !checkIfCandidateIsInValidRange(candidates.thirdCandidateYearInJst, candidates.thirdCandidateMonthInJst, candidates.thirdCandidateDayInJst, candidates.thirdCandidateHourInJst)) {
          errorBelowBtn.exists = true
          errorBelowBtn.message = `${Message.INVALID_CONSULTATION_DATE_TIME_MESSAGE}`
          return
        }

        let token: string
        try {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
          const createTokenResp: any = await payjp.createToken(cardElement)
          if (createTokenResp.error) {
            errorBelowBtn.exists = true
            errorBelowBtn.message = createTokenResp.error.message
            return
          }
          token = createTokenResp.id
        } catch (e) {
          errorBelowBtn.exists = true
          errorBelowBtn.message = `failed to create token: ${e}`
          return
        }

        try {
          startRequestConsultation()
          const req = {
            consultant_id: parseInt(consultantId),
            fee_per_hour_in_yen: feePerHourInYen.value,
            card_token: token,
            first_candidate_in_jst: {
              year: parseInt(candidates.firstCandidateYearInJst),
              month: parseInt(candidates.firstCandidateMonthInJst),
              day: parseInt(candidates.firstCandidateDayInJst),
              hour: parseInt(candidates.firstCandidateHourInJst)
            },
            second_candidate_in_jst: {
              year: parseInt(candidates.secondCandidateYearInJst),
              month: parseInt(candidates.secondCandidateMonthInJst),
              day: parseInt(candidates.secondCandidateDayInJst),
              hour: parseInt(candidates.secondCandidateHourInJst)
            },
            third_candidate_in_jst: {
              year: parseInt(candidates.thirdCandidateYearInJst),
              month: parseInt(candidates.thirdCandidateMonthInJst),
              day: parseInt(candidates.thirdCandidateDayInJst),
              hour: parseInt(candidates.thirdCandidateHourInJst)
            }
          } as ConsultationRequest
          try {
            const response = await postRequestConsultation(req)
            if (!(response instanceof PostRequestConsultationResp)) {
              if (!(response instanceof ApiErrorResp)) {
                throw new Error(`unexpected result on getting request detail: ${response}`)
              }
              const code = response.getApiError().getCode()
              if (code === Code.UNAUTHORIZED) {
                error.exists = true
                error.message = `${Message.UNAUTHORIZED_ON_CARD_OPERATION_MESSAGE}`
                return
              } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
                error.exists = true
                error.message = `${Message.NOT_TERMS_OF_USE_AGREED_YET_ON_CARD_OPERATION_MESSAGE}`
                return
              }
              error.exists = true
              error.message = createErrorMessage(response.getApiError().getCode())
              return
            }

            try {
              await payjp.openThreeDSecureDialog(response.getChargeId())
            } catch (e) {
              error.exists = true
              error.message = `${Message.UNEXPECTED_ERR}: ${e}`
              return
            }

            try {
              const finishRequestConsultation = {
                charge_id: response.getChargeId()
              } as FinishRequestConsultation
              const resp = await postFinishRequestConsultation(finishRequestConsultation)
              if (!(resp instanceof PostFinishRequestConsultationResp)) {
                if (!(resp instanceof ApiErrorResp)) {
                  throw new Error(`unexpected result on getting request detail: ${resp}`)
                }
                const code = resp.getApiError().getCode()
                if (code === Code.UNAUTHORIZED) {
                  error.exists = true
                  error.message = `${Message.UNAUTHORIZED_ON_CARD_OPERATION_MESSAGE}`
                  return
                } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
                  error.exists = true
                  error.message = `${Message.NOT_TERMS_OF_USE_AGREED_YET_ON_CARD_OPERATION_MESSAGE}`
                  return
                }
                error.exists = true
                error.message = createErrorMessage(resp.getApiError().getCode())
                return
              }
              await router.push('/request-consultation-success')
            } catch (e) {
              error.exists = true
              error.message = `${Message.UNEXPECTED_ERR}: ${e}`
            }
          } catch (e) {
            error.exists = true
            error.message = `${Message.UNEXPECTED_ERR}: ${e}`
          }
        } finally {
          finishRequestConsultation()
        }
      } finally {
        enableBtn()
      }
    }

    return {
      consultantId,
      error,
      getFeePerHourInYenForApplicationDone,
      feePerHourInYen,
      requestConsultationDone,
      disabled,
      yearList,
      monthList,
      dayList,
      hourList,
      minDurationInDays,
      maxDurationInDays,
      candidates,
      requestConsultation,
      errorBelowBtn
    }
  }
})
</script>
