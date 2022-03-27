<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">{{ message }}</h3>
      <div v-if="career !== null" class="font-bold text-lg">
        <div>career</div>
        <div>{{ career }}</div>
      </div>
      <p v-else class="font-bold text-lg">Not Found (career is null)</p>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import { useStore } from 'vuex'
import { Career } from '@/util/personalized/profile/Career'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'

export default defineComponent({
  name: 'EditCareerPage',
  components: {
    TheHeader
  },
  setup () {
    const message = ref('相談料編集用テストページ')
    const career = ref(null as Career | null)
    const route = useRoute()
    const router = useRouter()
    const store = useStore()
    onMounted(async () => {
      try {
        const resp = await refresh()
        if (resp instanceof RefreshResp) {
          // セッションが存在し、利用規約に同意済のため、ログイン後のページを表示可能
          // TODO: 正常系の処理
          // TODO: 実装メモ
          // store.state.careersのlengthが0 -> profileへ移動
          // idに一致するcareerがない -> Not Foundを表示 (TODO: そのようなケースがあるのか確認)
          const careers = store.state.careers
          const careerId = route.params.career_id as string
          career.value = findCareerById(careerId, careers)
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
    // router-linkで違うparamsを指定した際に備えてwatchを使う
    //  (TODO: そのようなケースがあるのか確認)
    watch(() => route.params.career_id, newId => {
      const careers = store.state.careers
      career.value = findCareerById(newId as string, careers)
    })
    return { message, career }
  }
})

function findCareerById (careerId: string, careers: Career[]): Career | null {
  for (const career of careers) {
    const careerIdStr = career.career_id.toString()
    if (careerIdStr === careerId) {
      return career
    }
  }
  return null
}
</script>
