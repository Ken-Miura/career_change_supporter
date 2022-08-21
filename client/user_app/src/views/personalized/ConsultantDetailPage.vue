<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">{{ message }}</h3>
      <h3 class="font-bold text-lg">{{ error.exists }} {{ error.message }}</h3>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, reactive, ref } from 'vue'
import { useRoute } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
// import { ApiErrorResp } from '@/util/ApiError'
// import { Code, createErrorMessage } from '@/util/Error'
// import { Message } from '@/util/Message'

export default defineComponent({
  name: 'ConsultantDetailPage',
  components: {
    TheHeader
  },
  setup () {
    const error = reactive({
      exists: false,
      message: ''
    })
    const message = ref('コンサルタント詳細用テストページ')
    // const router = useRouter()
    const route = useRoute()
    const consultantId = route.params.consultant_id as string
    onMounted(async () => {
      const params = { consultant_id: consultantId }
      const query = new URLSearchParams(params)
      const response = await fetch(`/api/consultant-detail?${query}`, {
        method: 'GET'
      })
      if (!response.ok) {
        error.exists = true
        error.message = 'error'
        return
      }
      message.value = await response.text()
      //  try {
      //   const resp = await refresh()
      //   if (!(resp instanceof RefreshResp)) {
      //     if (!(resp instanceof ApiErrorResp)) {
      //       throw new Error(`unexpected result on getting request detail: ${resp}`)
      //     }
      //     const code = resp.getApiError().getCode()
      //     if (code === Code.UNAUTHORIZED) {
      //       await router.push('/login')
      //       return
      //     } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
      //       await router.push('/terms-of-use')
      //       return
      //     }
      //     error.exists = true
      //     error.message = createErrorMessage(resp.getApiError().getCode())
      //   }
      // } catch (e) {
      //   error.exists = true
      //   error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      // }
    })
    return { error, message, consultantId }
  }
})
</script>
