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
          <h3 class="font-bold text-lg">{{ consultantsSearchResult.total }}</h3>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, onMounted, reactive, ref } from 'vue'
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
import { ConsultantsSearchResult } from '@/util/personalized/consultant-list/ConsultantsSearchResult'

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
    const consultantsSearchResult = ref({
      total: 0,
      consultants: []
    } as ConsultantsSearchResult)
    const searchParam = ref(null as ConsultantSearchParam | null)
    const currentPage = computed(() => {
      if (!searchParam.value) {
        return 0
      }
      return Math.floor(searchParam.value.from / searchParam.value.size)
    })
    const prevDisabled = computed(() => currentPage.value <= 0)
    const nextDisabled = computed(() => {
      if (!searchParam.value) {
        return true
      }
      return Math.floor(consultantsSearchResult.value.total / searchParam.value.size) >= currentPage.value
    })
    const router = useRouter()
    const store = useStore()
    onMounted(async () => {
      const consultantSearchParam = store.state.consultantSearchParam as ConsultantSearchParam | null
      searchParam.value = consultantSearchParam
      if (!searchParam.value) {
        error.exists = true
        error.message = Message.NO_CONSULTANT_SEARCH_PARAM_FOUND_MESSAGE
        return
      }
      try {
        const resp = await postConsultantsSearchFunc(searchParam.value)
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
        consultantsSearchResult.value = resp.getConsultantsSearchResult()
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })
    return {
      postConsultantsSearchDone,
      error,
      consultantsSearchResult,
      searchParam,
      currentPage,
      prevDisabled,
      nextDisabled
    }
  }
})
</script>
