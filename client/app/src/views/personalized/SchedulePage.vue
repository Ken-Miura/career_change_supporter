<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
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
import TheHeader from '@/components/TheHeader.vue'
import { checkAgreementStatus } from '@/util/personalized/agreement-status/CheckAgreementStatus'
import { CheckAgreementStatusResp } from '@/util/personalized/agreement-status/CheckAgreementStatusResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'

export default defineComponent({
  name: 'SchedulePage',
  components: {
    TheHeader
  },
  setup () {
    const message = ref('スケジュール用テストページ')
    const router = useRouter()
    onMounted(async () => {
      try {
        const agreementStatus = await checkAgreementStatus()
        if (agreementStatus instanceof CheckAgreementStatusResp) {
          // セッションが存在し、利用規約に同意済のため、ログイン後のページを表示可能
          // TODO: 正常系の処理
        } else if (agreementStatus instanceof ApiErrorResp) {
          const code = agreementStatus.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('terms-of-use')
            return
          }
          // TODO: エラー処理
        }
      } catch (e) {
        // TODO: エラー処理
      }
      console.log('TODO: 実装後削除')
    })
    return { message }
  }
})
</script>
