<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 bo min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">{{ message }}</h3>
      <button v-on:click="logoutHandler">ログアウト</button>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { refresh } from '@/util/refresh/Refresh'
import { ApiErrorResp } from '@/util/ApiError'
import { logout } from '@/util/logout/Logout'
import { LogoutResp } from '@/util/logout/LogoutResp'
import { CheckAgreementStatusResp } from '@/util/agreement-status/CheckAgreementStatusResp'
import { checkAgreementStatus } from '@/util/agreement-status/CheckAgreementStatus'
import { Code } from '@/util/Error'

export default defineComponent({
  name: 'Profile',
  setup () {
    const message = ref('プロファイル用テストページ')
    const router = useRouter()
    onMounted(async () => {
      try {
        const result = await refresh()
        if (!result) {
          await router.push('login')
          return
        }
        // セッションが存在するので、利用規約の確認
        const agreementStatus = await checkAgreementStatus()
        if (agreementStatus instanceof CheckAgreementStatusResp) {
          // セッションが存在し、利用規約に同意済のため、profileをそのまま表示する
        } else if (agreementStatus instanceof ApiErrorResp) {
          const code = agreementStatus.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('login')
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('terms-of-use')
          } else {
            throw new Error(`unexpected result: ${agreementStatus}`)
          }
        }
      } catch (e) {
        await router.push('login')
      }
    })
    const logoutHandler = async () => {
      try {
        const result = await logout()
        if (result instanceof LogoutResp) {
          console.log('LogoutResp')
        } else if (result instanceof ApiErrorResp) {
          console.log(`ApiErrorResp: ${result}`)
        } else {
          throw new Error(`unexpected result: ${result}`)
        }
      } catch (e) {
        console.log(`catch: ${e}`)
      }
      await router.push('login')
    }
    return { message, logoutHandler }
  }
})
</script>
