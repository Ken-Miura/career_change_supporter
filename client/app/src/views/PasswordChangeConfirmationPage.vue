<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="flex justify-center flex-col bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <button v-on:click="applyNewPasswordHandler" class="bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">パスワードを変更する</button>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { useRouter } from 'vue-router'
import { applyNewPassword } from '@/util/password/ApplyNewPassword'
import { ApplyNewPasswordResp } from '@/util/password/ApplyNewPasswordResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'
import { createErrorMessage } from '@/util/Error'
import { useStore } from '@/store/useStore'

export default defineComponent({
  name: 'NewPassword',
  setup () {
    const router = useRouter()
    const store = useStore()
    const applyNewPasswordHandler = async () => {
      const query = router.currentRoute.value.query
      const data = JSON.stringify(query)
      if (!data.match('"new-password-id"')) {
        store.commit('setApplyNewPasswordResultMessage', Message.INVALID_QUERY_PARAM)
        await router.push('apply-new-password-result')
        return
      }
      let message: string
      try {
        const result = await applyNewPassword(data)
        if (result instanceof ApplyNewPasswordResp) {
          message = Message.NEW_PASSWORD_APPLIED_MESSAGE
        } else if (result instanceof ApiErrorResp) {
          message = createErrorMessage(result.getApiError().getCode())
        } else {
          throw new Error(`unexpected result: ${result}`)
        }
      } catch (e) {
        message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
      store.commit('setApplyNewPasswordResultMessage', message)
      await router.push('apply-new-password-result')
    }
    return { applyNewPasswordHandler }
  }
})
</script>
