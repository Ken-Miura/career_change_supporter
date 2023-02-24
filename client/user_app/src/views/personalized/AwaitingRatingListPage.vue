<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">getAwaitingRatingsDone: {{ getAwaitingRatingsDone }}</h3>
      <h3 class="font-bold text-lg">errMessage: {{ errMessage }}</h3>
      <h3 class="font-bold text-lg">awaitingRatings: {{ awaitingRatings }}</h3>
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
import { useGetAwaitingRatings } from '@/util/personalized/awaiting-rating-list/useGetAwaitingRatings'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { AwaitingRatingsResp } from '@/util/personalized/awaiting-rating-list/AwaitingRatingsResp'
import { Message } from '@/util/Message'
import { AwaitingRatings } from '@/util/personalized/awaiting-rating-list/AwaitingRatings'

export default defineComponent({
  name: 'AwaitingRatingListPage',
  components: {
    TheHeader
  },
  setup () {
    const errMessage = ref(null as string | null)
    const router = useRouter()
    const {
      getAwaitingRatingsDone,
      getAwaitingRatingsFunc
    } = useGetAwaitingRatings()
    const awaitingRatings = ref(null as AwaitingRatings | null)

    onMounted(async () => {
      try {
        const resp = await getAwaitingRatingsFunc()
        if (!(resp instanceof AwaitingRatingsResp)) {
          if (!(resp instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${resp}`)
          }
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          }
          errMessage.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
        awaitingRatings.value = resp.getAwaitingRatings()
      } catch (e) {
        errMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })
    return {
      getAwaitingRatingsDone,
      errMessage,
      awaitingRatings
    }
  }
})
</script>
