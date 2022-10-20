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
          <h3 class="font-bold text-2xl">相談申し込み詳細</h3>
          <div class="m-4 text-2xl grid grid-cols-3">
            <div class="mt-2 justify-self-start col-span-2">コンサルタントID</div><div class="mt-2 justify-self-start col-span-1">{{ consultantId }}</div>
            <div class="mt-2 justify-self-start col-span-2">相談一回（１時間）の相談料</div><div class="mt-2 justify-self-start col-span-1">{{ feePerHourInYen }}円</div>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">相談開始日時（第一希望）</h3>
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option>10</option>
                <option>11</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              月
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option>1</option>
                <option>2</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              日
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option>7</option>
                <option>8</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              時
            </div>
          </div>
          <div class="mt-3" id="v2-demo"></div>
          <button class="mt-3" v-on:click="createToken">テスト</button>
          <div>{{ token }}</div>
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
        // elementsを取得します。ページ内に複数フォーム用意する場合は複数取得ください
        const elements = await payjp.elements()
        if (elements === null) {
          error.exists = true
          error.message = 'elements is null'
          return
        }
        // element(入力フォームの単位)を生成します
        cardElement = elements.create('card')
        if (cardElement === null) {
          error.exists = true
          error.message = 'cardElement is null'
          return
        }
        // elementをDOM上に配置します
        cardElement.mount('#v2-demo')
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    onUnmounted(async () => {
      cardElement.unmount()
      cardElement = null
    })

    const createToken = async () => {
      const payjp = store.state.payJp
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
      token,
      createToken
    }
  }
})
</script>
