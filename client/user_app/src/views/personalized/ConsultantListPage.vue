<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!postConsultantsSearchDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="error.exists" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <AlertMessage v-bind:message="error.message"/>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-lg">{{ message }}</h3>
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
import { useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { useStore } from 'vuex'
import { ConsultantSearchParam } from '@/util/personalized/ConsultantSearchParam'
import { usePostConsultantsSearch } from '@/util/personalized/consultant-list/usePostConsultantsSearch'
import { Message } from '@/util/Message'
import { PostConsultantsSearchResp } from '@/util/personalized/consultant-list/PostConsultantsSearchResp'

export default defineComponent({
  name: 'ConsultantListPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const {
      postConsultantsSearchDone,
      postConsultantsSearchFunc
    } = usePostConsultantsSearch()
    const error = reactive({
      exists: false,
      message: ''
    })
    const message = ref('相談者リスト用テストページ')
    const router = useRouter()
    const store = useStore()
    onMounted(async () => {
      const consultantSearchParam = store.state.consultantSearchParam as ConsultantSearchParam | null
      if (!consultantSearchParam) {
        error.exists = true
        error.message = 'null'
        return
      }
      try {
        const resp = await postConsultantsSearchFunc(consultantSearchParam)
        if (!(resp instanceof PostConsultantsSearchResp)) {
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
          error.exists = true
          error.message = createErrorMessage(resp.getApiError().getCode())
          return
        }
        const result = resp.getConsultantsSearchResult()
        message.value = `${result.total}: ${result.consultants}`
        console.log(result)
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })
    return { postConsultantsSearchDone, error, message }
  }
})
</script>
