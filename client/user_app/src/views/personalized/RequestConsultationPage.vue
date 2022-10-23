<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getFeePerHourInYenForApplicationDone" class="m-6">
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
          <p class="text-2xl">相談開始日時に関して、第一希望、第二希望、第三希望を入力して下さい。申し込み可能な相談開始日時は、申し込み日時から{{ minDurationInDays*24 }}時間（{{ minDurationInDays }}日）以降、{{ maxDurationInDays*24 }}時間（{{ maxDurationInDays }}日）以前までとなります。</p>
          <h3 class="mt-4 font-bold text-2xl">相談開始日時（第一希望）</h3>
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in yearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              年
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in monthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              月
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in dayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              日
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="hour in hourList" v-bind:key="hour" v-bind:value="hour">{{ hour }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              時
            </div>
          </div>
          <h3 class="mt-4 font-bold text-2xl">相談開始日時（第二希望）</h3>
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in yearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              年
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in monthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              月
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in dayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              日
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="hour in hourList" v-bind:key="hour" v-bind:value="hour">{{ hour }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              時
            </div>
          </div>
          <h3 class="mt-4 font-bold text-2xl">相談開始日時（第三希望）</h3>
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in yearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              年
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in monthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              月
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in dayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              日
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="hour in hourList" v-bind:key="hour" v-bind:value="hour">{{ hour }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              時
            </div>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">相談申し込み詳細</h3>
          <div class="m-4 text-2xl grid grid-cols-3">
            <div class="mt-2 justify-self-start col-span-2">コンサルタントID</div><div class="mt-2 justify-self-start col-span-1">{{ consultantId }}</div>
            <div class="mt-2 justify-self-start col-span-2">相談一回（１時間）の相談料</div><div class="mt-2 justify-self-start col-span-1">{{ feePerHourInYen }}円</div>
          </div>
          <h3 class="mt-4 font-bold text-2xl">クレジットカード</h3>
          <div class="m-4 text-2xl flex flex-col">
            <div class="mt-2 w-5/6" id="payjp-card-area"></div>
            <div class="mt-2 w-5/6">{{ token }}</div>
          </div>
          <button class="mt-8 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200" v-on:click="requestConsultation">相談を申し込む</button>
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
    // PAY.JPから型定義が提供されていないため、anyでの扱いを許容する
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let cardElement = null as any
    const token = ref('')
    const {
      getFeePerHourInYenForApplicationDone,
      getFeePerHourInYenForApplicationFunc
    } = useGetFeePerHourInYenForApplication()
    const feePerHourInYen = ref(null as number | null)

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
      const payjp = store.state.payJp
      if (payjp === null) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: payjp is null`
        return
      }
      try {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const r: any = await payjp.createToken(cardElement)
        if (r.error) {
          token.value = r.error.message
          return
        }
        token.value = r.id
      } catch (e) {
        token.value = `failed to createToken: ${e}`
        return
      }
      const data = {
        consultant_id: parseInt(consultantId),
        fee_per_hour_in_yen: 3000,
        card_token: token.value,
        first_candidate_in_jst: {
          year: 2022,
          month: 10,
          day: 22,
          hour: 8
        },
        second_candidate_in_jst: {
          year: 2022,
          month: 10,
          day: 22,
          hour: 12
        },
        third_candidate_in_jst: {
          year: 2022,
          month: 10,
          day: 22,
          hour: 15
        }
      }
      const response = await fetch('/api/request-consultation', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json; charset=utf-8' },
        body: JSON.stringify(data)
      })
      if (!response.ok) {
        const apiErr = await response.json() as { code: number }
        console.error(response.status + ', ' + apiErr.code)
        return
      }
      const result = await response.json() as { charge_id: string }
      await payjp.openThreeDSecureDialog(result.charge_id)
      const resp = await fetch('/api/finish-request-consultation', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json; charset=utf-8' },
        body: JSON.stringify(result)
      })
      if (!resp.ok) {
        const apiErr = await resp.json() as { code: number }
        console.error(resp.status + ', ' + apiErr.code)
      }
    }

    return {
      consultantId,
      error,
      getFeePerHourInYenForApplicationDone,
      feePerHourInYen,
      yearList,
      monthList,
      dayList,
      hourList,
      minDurationInDays,
      maxDurationInDays,
      token,
      requestConsultation
    }
  }
})
</script>
