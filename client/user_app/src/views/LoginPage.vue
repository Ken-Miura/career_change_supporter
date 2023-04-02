<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <div v-if="!loginDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else class="bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <section>
        <h3 class="font-bold text-2xl">ログイン</h3>
      </section>
      <section class="mt-10">
        <form class="flex flex-col" @submit.prevent="loginHandler">
          <EmailAddressInput class="mb-6" @on-email-address-updated="setEmailAddress"/>
          <PasswordInput class="mb-6" @on-password-updated="setPassword" label="パスワード"/>
          <div class="flex justify-end">
            <router-link to="/password-change-req" class="text-sm text-gray-600 hover:text-gray-700 hover:underline mb-6">パスワードを忘れた、または変更したい場合</router-link>
          </div>
          <button class="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">ログイン</button>
          <AlertMessage v-bind:class="['mt-6', { 'hidden': isHidden }]" v-bind:message="errorMessage"/>
        </form>
      </section>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import EmailAddressInput from '@/components/EmailAddressInput.vue'
import PasswordInput from '@/components/PasswordInput.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { useRouter } from 'vue-router'
import { useCredentil } from '@/components/useCredential'
import { Message } from '@/util/Message'
import { Code, createErrorMessage } from '@/util/Error'
import { ApiErrorResp } from '@/util/ApiError'
import { LoginResp } from '@/util/login/LoginResp'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { useLogin } from '@/util/login/useLogin'
import WaitingCircle from '@/components/WaitingCircle.vue'

export default defineComponent({
  name: 'LoginPage',
  components: {
    EmailAddressInput,
    PasswordInput,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()
    const isHidden = ref(true)
    const errorMessage = ref('')

    onMounted(async () => {
      try {
        const resp = await refresh()
        if (resp instanceof RefreshResp) {
          await router.push('/profile')
          return
        } else if (resp instanceof ApiErrorResp) {
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            // ログインセッションが存在しないため、そのままログインページを表示する
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          } else {
            throw new Error(`unexpected result: ${resp}`)
          }
        }
      } catch (e) {
        isHidden.value = false
        errorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const {
      form,
      setEmailAddress,
      setPassword
    } =
    useCredentil()
    const {
      loginDone,
      loginFunc
    } = useLogin()

    const loginHandler = async () => {
      try {
        const result = await loginFunc(form.emailAddress, form.password)
        if (!(result instanceof LoginResp)) {
          if (!(result instanceof ApiErrorResp)) {
            throw new Error(`unexpected result: ${result}`)
          }
          isHidden.value = false
          errorMessage.value = createErrorMessage(result.getApiError().getCode())
          return
        }
        const ls = result.getLoginResult()
        if (ls.login_status === 'Finish') {
          await router.push('/profile')
        } else if (ls.login_status === 'NeedMoreVerification') {
          await router.push('/mfa')
        } else {
          throw new Error(`unexpected login_status: ${ls}`)
        }
      } catch (e) {
        isHidden.value = false
        errorMessage.value = `${Message.LOGIN_FAILED}: ${e}`
      }
    }
    return {
      form,
      setEmailAddress,
      setPassword,
      isHidden,
      errorMessage,
      loginHandler,
      loginDone
    }
  }
})
</script>
