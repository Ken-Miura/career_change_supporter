<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="flex flex-col justify-center bg-white max-w-3xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <TermsOfUse/>
      <div class="flex justify-center mt-6">
        <button v-on:click="agreeTermsOfUseHandler" class="bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">利用規約に同意する</button>
      </div>
      <AlertMessage v-bind:class="['mt-6', { 'hidden': isHidden }]" v-bind:message="errorMessage"/>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ApiErrorResp } from '@/util/ApiError'
import { agreeTermsOfUse } from '@/util/personalized/terms-of-use/AgreeTermsOfUse'
import { AgreeTermsOfUseResp } from '@/util/personalized/terms-of-use/AgreeTermsOfUseResp'
import { Code } from '@/util/Error'
import AlertMessage from '@/components/AlertMessage.vue'
import TermsOfUse from '@/components/TermsOfUse.vue'
import { Message } from '@/util/Message'

export default defineComponent({
  name: 'TermsOfUsePage',
  components: {
    AlertMessage,
    TermsOfUse
  },
  setup () {
    const router = useRouter()
    const isHidden = ref(true)
    const errorMessage = ref('')
    onMounted(async () => {
      // NOTE:
      // ログインセッションの延長のためのリクエストは実施しない。<br>
      // このページに遷移してくるのは、ログインセッションがあることが確認でき、利用規約が同意されていなかった場合である。
      // ログインセッションがあることが確認できた際にログインセッションの延長がされているため、ここでわざわざログインセッション延長のためのリクエストを送る必要がない。
    })
    const agreeTermsOfUseHandler = async () => {
      try {
        const result = await agreeTermsOfUse()
        if (result instanceof AgreeTermsOfUseResp) {
          await router.push('profile')
          return
        } else if (result instanceof ApiErrorResp) {
          const code = result.getApiError().getCode()
          // 利用規約に同意するためにはセッションが有効である必要がある
          // そのため、セッションの期限が切れている場合、Code.UNAUTHORIZEDが返却される
          if (code === Code.UNAUTHORIZED) {
            await router.push('login')
            return
          } else if (code === Code.ALREADY_AGREED_TERMS_OF_USE) {
            // 複数回連続で利用規約に同意するを押した場合、Code.ALREADY_AGREED_TERMS_OF_USEが返却される可能性が考えられる
            // 既に利用規約に同意している場合は、無視してprofile画面へ遷移する
            await router.push('profile')
            return
          } else {
            throw new Error(`unexpected result: ${result}`)
          }
        } else {
          throw new Error(`unexpected result: ${result}`)
        }
      } catch (e) {
        isHidden.value = false
        errorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }
    return { isHidden, errorMessage, agreeTermsOfUseHandler }
  }
})
</script>
