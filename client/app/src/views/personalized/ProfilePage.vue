<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 bo min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">{{ message }}</h3>
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
import { CheckAgreementStatusResp } from '@/util/agreement-status/CheckAgreementStatusResp'
import { checkAgreementStatus } from '@/util/agreement-status/CheckAgreementStatus'
import { Code } from '@/util/Error'
import TheHeader from '@/components/TheHeader.vue'

export default defineComponent({
  name: 'ProfilePage',
  components: {
    TheHeader
  },
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
    return { message }
  }
})
</script>
