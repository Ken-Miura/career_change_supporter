<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <div v-if="!createTempAccountDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else class="bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <section>
        <h3 class="font-bold text-2xl">新規登録</h3>
      </section>
      <section class="mt-10">
        <form class="flex flex-col" @submit.prevent="createTempAccountHandler">
          <EmailAddressInput class="mb-6" @on-email-address-updated="setEmailAddress"/>
          <PasswordInput class="mb-6" @on-password-updated="setPassword" label="パスワード"/>
          <PasswordInput class="mb-6" @on-password-updated="setPasswordConfirmation" label="パスワード（確認）"/>
          <button class="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">新規登録</button>
          <AlertMessage v-bind:class="['mt-6', { 'hidden': isHidden }]" v-bind:message="errorMessage"/>
        </form>
      </section>
      <div>
        <div class="m-2 text-2xl">
          <div class="mt-2">テスト</div>
          <img class="mt-2" v-bind:src="imageRef" />
        </div>
        <div>
          <div class="pt-3 rounded bg-gray-200">
            <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">コード</label>
            <input v-model="codeInput" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
          </div>
          <button v-on:click="submitCode" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">コード送信</button>
        </div>
        <div v-if="status">
          {{ status }}
        </div>
      </div>
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
import { useCredentil } from '@/components/useCredential'
import { useRouter } from 'vue-router'
import { ApiErrorResp } from '@/util/ApiError'
import { CreateTempAccountResp } from '@/util/temp-account/CreateTempAccountResp'
import { createErrorMessage } from '@/util/Error'
import { Message } from '@/util/Message'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useCreateTempAccount } from '@/util/temp-account/useCreateTempAccount'

export default defineComponent({
  name: 'NewAccountPage',
  components: {
    EmailAddressInput,
    PasswordInput,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()
    const {
      form,
      setEmailAddress,
      setPassword,
      setPasswordConfirmation,
      passwordsAreSame
    } =
    useCredentil()
    const isHidden = ref(true)
    const errorMessage = ref('')
    const {
      createTempAccountDone,
      createTempAccountFunc
    } = useCreateTempAccount()

    const createTempAccountHandler = async () => {
      if (!passwordsAreSame.value) {
        isHidden.value = false
        errorMessage.value = Message.PASSWORD_CONFIRMATION_FAILED
        return
      }
      try {
        // 新規作成の流れは下記の通り
        // 1. システムは、一時アカウントを作成し、ユーザーにメールを送信する
        // 2. ユーザーは、メールに記載してあるURLにアクセスし、アカウントを新規作成する
        // 下記の関数では1の機能を提供する
        const result = await createTempAccountFunc(form.emailAddress, form.password)
        if (result instanceof CreateTempAccountResp) {
          await router.push('/temp-account-creation-result')
          return
        } else if (result instanceof ApiErrorResp) {
          isHidden.value = false
          errorMessage.value = createErrorMessage(result.getApiError().getCode())
        } else {
          throw new Error(`unexpected result: ${result}`)
        }
      } catch (e) {
        isHidden.value = false
        errorMessage.value = `${Message.TEMP_ACCOUNT_CREATION_FAILED}: ${e}`
      }
    }

    const imageRef = ref('')
    onMounted(async () => {
      const resp = await fetch('/api/mfa')
      const result = await resp.json() as { qr: string }
      imageRef.value = `data:image/png;base64,${result.qr}`
    })

    const codeInput = ref('')
    const status = ref(null as number | null)

    const submitCode = async () => {
      const data = { code: codeInput.value }
      const resp = await fetch('/api/check', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json; charset=utf-8' },
        body: JSON.stringify(data)
      })
      status.value = resp.status
    }

    return {
      form,
      setEmailAddress,
      setPassword,
      setPasswordConfirmation,
      isHidden,
      errorMessage,
      createTempAccountHandler,
      createTempAccountDone,
      imageRef,
      codeInput,
      submitCode,
      status
    }
  }
})
</script>
