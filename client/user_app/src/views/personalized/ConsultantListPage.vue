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
      <div v-else class="flex flex-col justify-center max-w-4xl mx-auto">
        <div class="grid grid-cols-2 max-w-4xl">
          <div class="justify-self-start ml-2 col-span-1">
            <div class="bg-white text-xl px-6 py-4 rounded-lg shadow-2xl">{{ consultantsSearchResult.total }} 件</div>
          </div>
          <div class="justify-self-end mr-2 col-span-1">
            <div class="grid grid-cols-3 items-center bg-white text-xl px-4 py-2 rounded-lg shadow-2xl">
              <div class="col-span-1">ソート：</div>
              <select v-model="sortParam" v-on:change="onSortParamChanged" class="col-span-2 block p-3 w-full rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option value="none">指定なし</option>
                <option value="fee_asc">相談料が安い順</option>
                <option value="fee_desc">相談料が高い順</option>
                <option value="rating_desc">評価が高い順</option>
                <option value="rating_asc">評価が安い順</option>
              </select>
            </div>
          </div>
        </div>
        <div class="bg-white p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-lg">{{ consultantsSearchResult.consultants }}</h3>
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
import { ConsultantSearchParam, SortParam } from '@/util/personalized/ConsultantSearchParam'
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
    const firstPage = 0
    const lastPage = computed(() => {
      if (!searchParam.value) {
        return firstPage
      }
      return Math.floor(consultantsSearchResult.value.total / searchParam.value.size)
    })
    const pageSelection = computed(() => {
      const pageSize = 2
      if (!searchParam.value) {
        return []
      }
      const min = Math.max(0, currentPage.value - pageSize)
      const lastPage = Math.floor(consultantsSearchResult.value.total / searchParam.value.size)
      const max = Math.min(lastPage, currentPage.value + pageSize)
      const pageSelection = []
      for (let i = min; i < max; i++) {
        pageSelection.push(i)
      }
      return pageSelection
    })
    const sortParam = ref('none')
    const router = useRouter()
    const store = useStore()

    const searchConsultants = async (searchParam: ConsultantSearchParam) => {
      try {
        const resp = await postConsultantsSearchFunc(searchParam)
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
    }

    onMounted(async () => {
      const consultantSearchParam = store.state.consultantSearchParam as ConsultantSearchParam | null
      searchParam.value = consultantSearchParam
      if (!searchParam.value) {
        error.exists = true
        error.message = Message.NO_CONSULTANT_SEARCH_PARAM_FOUND_MESSAGE
        return
      }
      searchConsultants(searchParam.value)
    })

    const onSortParamChanged = async () => {
      if (!searchParam.value) {
        error.exists = true
        error.message = Message.NO_CONSULTANT_SEARCH_PARAM_FOUND_MESSAGE
        return
      }
      try {
        searchParam.value.sort_param = generateSortParam(sortParam.value)
        searchConsultants(searchParam.value)
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    return {
      postConsultantsSearchDone,
      error,
      consultantsSearchResult,
      searchParam,
      currentPage,
      firstPage,
      lastPage,
      pageSelection,
      sortParam,
      onSortParamChanged
    }
  }
})

function generateSortParam (sort: string): SortParam | null {
  if (sort === 'none') {
    return null
  } else if (sort === 'fee_asc') {
    return {
      key: 'fee_per_hour_in_yen',
      order: 'asc'
    } as SortParam
  } else if (sort === 'fee_desc') {
    return {
      key: 'fee_per_hour_in_yen',
      order: 'desc'
    } as SortParam
  } else if (sort === 'rating_desc') {
    return {
      key: 'rating',
      order: 'desc'
    } as SortParam
  } else if (sort === 'rating_asc') {
    return {
      key: 'rating',
      order: 'asc'
    } as SortParam
  } else {
    throw new Error('invalid sort parameter')
  }
}
</script>
