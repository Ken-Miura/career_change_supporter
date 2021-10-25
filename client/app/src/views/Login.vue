<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 bo min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <section>
        <h3 class="font-bold text-2xl">ログイン</h3>
      </section>
      <section class="mt-10">
        <form class="flex flex-col" @submit.prevent="loginHandler">
          <EmailAddress class="mb-6" @on-email-address-updated="setEmailAddress"/>
          <Password class="mb-6" @on-password-updated="setPassword" label="パスワード"/>
          <div class="flex justify-end">
            <router-link to="/password-change" class="text-sm text-gray-600 hover:text-gray-700 hover:underline mb-6">パスワードを忘れた、または変更したい場合</router-link>
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
import EmailAddress from '@/components/EmailAddress.vue'
import Password from '@/components/Password.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { useRouter } from 'vue-router'
import { useCredentil } from '@/components/useCredential'
import { Message } from '@/util/Message'
import { createErrorMessage } from '@/util/Error'
import { ApiErrorResp } from '@/util/ApiError'
import { LoginResp } from '@/util/login/LoginResp'
import { login } from '@/util/login/Login'
import { refresh } from '@/util/refresh/Refresh'

export default defineComponent({
  name: 'Login',
  components: {
    EmailAddress,
    Password,
    AlertMessage
  },
  setup () {
    const router = useRouter()
    onMounted(async () => {
      try {
        const result = await refresh()
        if (result === 'SUCCESS') {
          await router.push('profile')
        } else if (result === 'FAILURE') {
          // refreshに失敗 => セッションが切れている => ログイン画面へ遷移となる
          // ただ、もともとログインページなのでrouteを更新する必要はない。なので何もしない
        } else {
          throw new Error(`unexpected result: ${result}`)
        }
      } catch (e) {
        await router.push('login')
      }
    })
    const {
      form,
      setEmailAddress,
      setPassword
    } =
    useCredentil()
    const isHidden = ref(true)
    const errorMessage = ref('')
    const loginHandler = async () => {
      try {
        const result = await login(form.emailAddress, form.password)
        if (result instanceof LoginResp) {
          await router.push('profile')
        } else if (result instanceof ApiErrorResp) {
          isHidden.value = false
          errorMessage.value = createErrorMessage(result.getApiError().getCode())
        } else {
          throw new Error(`unexpected result: ${result}`)
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
      loginHandler
    }
  }
})
</script>
