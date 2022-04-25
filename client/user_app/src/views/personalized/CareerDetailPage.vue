<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">{{ message }}</h3>
      <div class="font-bold text-lg">
        <p>careerId: {{ careerId }}</p>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'

export default defineComponent({
  name: 'CareerDetailPage',
  components: {
    TheHeader
  },
  setup () {
    const message = ref('職務経歴詳細テストページ')
    const careerId = ref('' as string)
    const route = useRoute()
    const router = useRouter()
    onMounted(async () => {
      try {
        const resp = await refresh()
        if (resp instanceof RefreshResp) {
          // セッションが存在し、利用規約に同意済のため、ログイン後のページを表示可能
          // TODO: 正常系の処理
          // TODO: 実装メモ
          careerId.value = route.params.career_id as string
        } else if (resp instanceof ApiErrorResp) {
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
          }
          // TODO: エラー処理
        }
      } catch (e) {
        // TODO: エラー処理
      }
    })
    return { message, careerId }
  }
})
</script>
