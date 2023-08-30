<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!postFeePerHourInYenDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-xl lg:text-2xl">相談一回（１時間）の相談料</h3>
      <p class="mt-2 text-base lg:text-lg">相談一回（１時間）の相談料には、{{ MIN_FEE_PER_HOUR_IN_YEN }}円以上、{{ MAX_FEE_PER_HOUR_IN_YEN }}円以下の値を設定出来ます。</p>
      <form @submit.prevent="submitFeePerHourInYen">
        <div class="m-4 text-xl lg:text-2xl grid grid-cols-6">
          <div data-test="fee-input-div" class="mt-2 min-w-full justify-self-start col-span-5 pt-3 rounded bg-gray-200">
            <input v-bind:value="feePerHourInYen" v-on:input="setFeePerHourInYen" type="text" minlength="4" maxlength="5" pattern="\d*" title="半角数字でご入力下さい。" class="text-right bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
          </div>
          <div class="mt-2 ml-4 min-w-full justify-self-start col-span-1 pt-3">
            円
          </div>
        </div>
        <button data-test="submit-button" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">相談料を設定する</button>
        <AlertMessage v-bind:class="['mt-6', { 'hidden': !error.exists }]" v-bind:message="error.message"/>
      </form>
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
import { useStore } from 'vuex'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { Message } from '@/util/Message'
import { usePostFeePerHourInYen } from '@/util/personalized/fee-per-hour-in-yen/usePostFeePerHourInYen'
import { PostFeePerHourInYenResp } from '@/util/personalized/fee-per-hour-in-yen/PostFeePerHourInYenResp'
import { MIN_FEE_PER_HOUR_IN_YEN, MAX_FEE_PER_HOUR_IN_YEN } from '@/util/Fee'

export default defineComponent({
  name: 'FeePerHourInYenPage',
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
    const feePerHourInYen = ref(null as string | null)
    const router = useRouter()
    const store = useStore()
    onMounted(async () => {
      try {
        const resp = await refresh()
        if (!(resp instanceof RefreshResp)) {
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
        const value = store.state.feePerHourInYen
        feePerHourInYen.value = value ? value.toString() : ''
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const setFeePerHourInYen = (e: Event) => {
      const target = (e && e.target)
      if (!(target instanceof HTMLInputElement)) {
        throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
      }
      feePerHourInYen.value = target.value
    }

    const {
      postFeePerHourInYenDone,
      postFeePerHourInYenFunc
    } = usePostFeePerHourInYen()

    const submitFeePerHourInYen = async () => {
      if (!feePerHourInYen.value) {
        // inputタグのminlength属性で空文字を拒否している。
        // 空文字かどうかチェックしてエラーを表示するのは上記のロジックと重複するので特にエラー表示はしない。
        return
      }
      try {
        const feaPerHourInYen = parseInt(feePerHourInYen.value)
        if (feaPerHourInYen < MIN_FEE_PER_HOUR_IN_YEN || feaPerHourInYen > MAX_FEE_PER_HOUR_IN_YEN) {
          error.exists = true
          error.message = Message.ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE
          return
        }
        const response = await postFeePerHourInYenFunc(feaPerHourInYen)
        if (!(response instanceof PostFeePerHourInYenResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          }
          error.exists = true
          error.message = createErrorMessage(response.getApiError().getCode())
          return
        }
        await router.push('/submit-fee-per-hour-in-yen-success')
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    return {
      postFeePerHourInYenDone,
      error,
      feePerHourInYen,
      setFeePerHourInYen,
      submitFeePerHourInYen,
      MIN_FEE_PER_HOUR_IN_YEN,
      MAX_FEE_PER_HOUR_IN_YEN
    }
  }
})
</script>
