<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 data-test="header" class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <div v-if="!postRecoveryCodeDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else class="bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <section>
        <h3 data-test="login-label" class="font-bold text-2xl">ログイン</h3>
      </section>
      <p data-test="login-description" class="mt-2 ml-2">二段階認証設定時に保存したリカバリーコードを入力して下さい。リカバリーコードによるログイン後、二段階認証の設定は無効化されますので、適宜再設定を行うようお願いします。</p>
      <section class="mt-6">
        <form class="flex flex-col" @submit.prevent="recoveryCodeHandler">
          <div class="mt-2 mb-6 w-full justify-self-start col-span-6 pt-3 pl-2 rounded bg-gray-200">
            <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">リカバリーコード</label>
            <input data-test="recovery-code-input" v-model="recoveryCode" type="text" pattern="[a-zA-Z0-9]{32}" title="半角英数字のみの32桁でご入力下さい。" required minlength="32" maxlength="32" class="text-xl text-right bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
          </div>
          <div data-test="pass-code-link-area" class="flex justify-end">
            <router-link to="/mfa" class="text-sm text-gray-600 hover:text-gray-700 hover:underline mb-6">認証アプリ（パスコード）を用いたログイン</router-link>
          </div>
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
import AlertMessage from '@/components/AlertMessage.vue'
import { useRouter } from 'vue-router'
import { Message } from '@/util/Message'
import { Code, createErrorMessage } from '@/util/Error'
import { ApiErrorResp } from '@/util/ApiError'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { usePostRecoveryCode } from '@/util/mfa/usePostRecoveryCode'
import { PostRecoveryCodeResp } from '@/util/mfa/PostRecoveryCodeResp'

export default defineComponent({
  name: 'RecoveryCodePage',
  components: {
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()
    const errorMessage = ref(null as string | null)

    const recoveryCode = ref('')

    const {
      postRecoveryCodeDone,
      postRecoveryCodeFunc
    } = usePostRecoveryCode()

    const recoveryCodeHandler = async () => {
      try {
        const resp = await postRecoveryCodeFunc(recoveryCode.value)
        if (!(resp instanceof PostRecoveryCodeResp)) {
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
        await router.push('/profile')
      } catch (e) {
        errorMessage.value = `${Message.LOGIN_FAILED}: ${e}`
      }
    }

    return {
      postRecoveryCodeDone,
      recoveryCodeHandler,
      recoveryCode,
      errorMessage
    }
  }
})
</script>
