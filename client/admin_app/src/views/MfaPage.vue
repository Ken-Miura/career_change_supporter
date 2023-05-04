<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 data-test="header" class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <div v-if="!postPassCodeDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else class="bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <section>
        <h3 data-test="login-label" class="font-bold text-2xl">ログイン</h3>
      </section>
      <p data-test="login-description" class="mt-2 ml-2">認証アプリに表示されているパスコード（6桁の数字）を入力して下さい。</p>
      <section class="mt-6">
        <form class="flex flex-col" @submit.prevent="passCodeHandler">
          <PassCodeInput class="mb-6" @on-pass-code-updated="setPassCode" label="パスコード"/>
          <button data-test="login-button" class="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">ログイン</button>
          <div v-if="errorMessage" class="mt-6">
            <AlertMessage v-bind:message="errorMessage"/>
          </div>
        </form>
      </section>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import PassCodeInput from '@/components/PassCodeInput.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { useRouter } from 'vue-router'
import { Message } from '@/util/Message'
import { Code, createErrorMessage } from '@/util/Error'
import { ApiErrorResp } from '@/util/ApiError'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { usePassCode } from '@/components/usePassCode'
import { usePostPassCode } from '@/util/mfa/usePostPassCode'
import { PostPassCodeResp } from '@/util/mfa/PostPassCodeResp'

export default defineComponent({
  name: 'MfaPage',
  components: {
    PassCodeInput,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()
    const errorMessage = ref(null as string | null)

    const {
      passCode,
      setPassCode
    } = usePassCode()

    const {
      postPassCodeDone,
      postPassCodeFunc
    } = usePostPassCode()

    const passCodeHandler = async () => {
      try {
        const resp = await postPassCodeFunc(passCode.value)
        if (!(resp instanceof PostPassCodeResp)) {
          if (!(resp instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${resp}`)
          }
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          errorMessage.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
        await router.push('/admin-menu')
      } catch (e) {
        errorMessage.value = `${Message.LOGIN_FAILED}: ${e}`
      }
    }

    return {
      postPassCodeDone,
      passCodeHandler,
      setPassCode,
      errorMessage
    }
  }
})
</script>
