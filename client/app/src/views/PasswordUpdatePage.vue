<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="flex justify-center flex-col bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <section>
        <h3 class="font-bold text-2xl">パスワード変更</h3>
      </section>
      <section class="mt-10">
        <form class="flex flex-col" @submit.prevent="updatePasswordHandler">
          <PasswordInput class="mb-6" @on-password-updated="setPassword" label="新しいパスワード"/>
          <PasswordInput class="mb-6" @on-password-updated="setPasswordConfirmation" label="新しいパスワード（確認）"/>
          <button class="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">パスワード変更</button>
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
import { useRouter } from 'vue-router'
import PasswordInput from '@/components/PasswordInput.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { updatePassword } from '@/util/password/UpdatePassword'
import { UpdatePasswordResp } from '@/util/password/UpdatePasswordResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'
import { createErrorMessage } from '@/util/Error'
import { useStore } from 'vuex'
import { SET_PASSWORD_UPDATE_RESULT_MESSAGE } from '@/store/mutationTypes'
import { useCredentil } from '@/components/useCredential'

export default defineComponent({
  name: 'PasswordUpdatePage',
  components: {
    PasswordInput,
    AlertMessage
  },
  setup () {
    const router = useRouter()
    const store = useStore()
    const {
      form,
      setPassword,
      setPasswordConfirmation,
      passwordsAreSame
    } =
    useCredentil()
    const isHidden = ref(true)
    const errorMessage = ref('')
    const updatePasswordHandler = async () => {
      const query = router.currentRoute.value.query
      const pwdChangeReqId = query['pwd-change-req-id'] as string
      if (!pwdChangeReqId) {
        isHidden.value = false
        errorMessage.value = Message.INVALID_QUERY_PARAM
        return
      }
      if (!passwordsAreSame.value) {
        isHidden.value = false
        errorMessage.value = Message.PASSWORD_CONFIRMATION_FAILED
        return
      }
      let message: string
      try {
        const result = await updatePassword(pwdChangeReqId, form.password)
        if (result instanceof UpdatePasswordResp) {
          message = Message.PASSWORD_CHANGED_MESSAGE
        } else if (result instanceof ApiErrorResp) {
          message = createErrorMessage(result.getApiError().getCode())
        } else {
          throw new Error(`unexpected result: ${result}`)
        }
      } catch (e) {
        message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
      store.commit(SET_PASSWORD_UPDATE_RESULT_MESSAGE, message)
      await router.push('password-update-result')
    }
    return { form, setPassword, setPasswordConfirmation, isHidden, errorMessage, updatePasswordHandler }
  }
})
</script>
