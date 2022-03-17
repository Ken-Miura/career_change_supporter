<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <section>
        <h3 class="font-bold text-2xl">パスワード変更依頼</h3>
      </section>
      <section class="mt-10">
        <form class="flex flex-col" @submit.prevent="createPwdChangeReqHandler">
          <EmailAddressInput class="mb-6" @on-email-address-updated="setEmailAddress"/>
          <button class="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">パスワード変更依頼</button>
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
import { defineComponent, ref } from 'vue'
import EmailAddressInput from '@/components/EmailAddressInput.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { Message } from '@/util/Message'
import { useRouter } from 'vue-router'
import { useCredentil } from '@/components/useCredential'
import { createErrorMessage } from '@/util/Error'
import { ApiErrorResp } from '@/util/ApiError'
import { createPwdChangeReq } from '@/util/password/CreatePwdChangeReq'
import { CreatePwdChangeReqResp } from '@/util/password/CreatePwdChangeReqResp'

export default defineComponent({
  name: 'PasswordChangeReqPage',
  components: {
    EmailAddressInput,
    AlertMessage
  },
  setup () {
    const router = useRouter()
    const {
      form,
      setEmailAddress
    } =
    useCredentil()
    const isHidden = ref(true)
    const errorMessage = ref('')
    const createPwdChangeReqHandler = async () => {
      try {
        const result = await createPwdChangeReq(form.emailAddress)
        if (result instanceof CreatePwdChangeReqResp) {
          await router.push('password-change-req-result')
          return
        } else if (result instanceof ApiErrorResp) {
          isHidden.value = false
          errorMessage.value = createErrorMessage(result.getApiError().getCode())
        } else {
          throw new Error(`unexpected result: ${result}`)
        }
      } catch (e) {
        isHidden.value = false
        errorMessage.value = `${Message.PASSWORD_CHANGE_REQUEST_FAILED}: ${e}`
      }
    }
    return {
      form,
      setEmailAddress,
      isHidden,
      errorMessage,
      createPwdChangeReqHandler
    }
  }
})
</script>
