<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="false" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="error.exists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="error.message"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">相談申し込み詳細</h3>
          <p class="mt-2 text-lg">詳細を確認し、相談申し込みを受けるかどうか選択して下さい。</p>
          <div class="grid grid-cols-2 mt-4 ml-4">
            <h3 class="text-xl justify-self-start col-span-1">ユーザーID</h3><h3 class="text-xl justify-self-start col-span-1">2</h3>
            <div class="mt-3 justify-self-start col-span-1 text-xl">評価</div><div class="mt-3 justify-self-start col-span-1 text-xl"><span v-if="true"> 4.5</span><span v-else>0</span>/5（評価件数：21 件）</div>
            <p class="mt-3 justify-self-start col-span-1 text-xl">支払われる相談料</p><p class="mt-3 justify-self-start col-span-1 text-xl">5000 円</p>
          </div>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'

export default defineComponent({
  name: 'ConsultationRequestDetailPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const error = reactive({
      exists: false,
      message: ''
    })
    const router = useRouter()
    const route = useRoute()
    const consultationReqId = route.params.consultation_req_id as string
    onMounted(async () => {
      try {
        const resp = await refresh()
        if (resp instanceof RefreshResp) {
          // セッションが存在し、利用規約に同意済のため、ログイン後のページを表示可能
          // TODO: 正常系の処理
        } else if (resp instanceof ApiErrorResp) {
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          }
          // TODO: エラー処理
        }
      } catch (e) {
        // TODO: エラー処理
      }
      console.log('TODO: 実装後削除')
    })
    return { error }
  }
})
</script>
