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
          <div data-test="total" class="justify-self-start ml-2 col-span-1">
            <div class="bg-white text-xl px-6 py-4 rounded-lg shadow-2xl">{{ consultantsSearchResult.total }} 件</div>
          </div>
          <div class="justify-self-end mr-2 col-span-1">
            <div class="grid grid-cols-3 items-center bg-white text-xl px-4 py-2 rounded-lg shadow-2xl">
              <div data-test="sort-label" class="col-span-1">ソート：</div>
              <select data-test="sort-value" v-model="sortParam" v-on:change="onSortParamChanged" class="col-span-2 block p-3 w-full rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option value="none">指定なし</option>
                <option value="fee_asc">相談料が安い順</option>
                <option value="fee_desc">相談料が高い順</option>
                <option value="rating_desc">評価が高い順</option>
                <option value="rating_asc">評価が安い順</option>
              </select>
            </div>
          </div>
        </div>
        <div data-test="consultants-area" class="flex flex-col justify-center my-5">
          <div data-test="consultant" v-for="consultant in consultantsSearchResult.consultants" v-bind:key="consultant.consultant_id" class="bg-white p-8 md:p-12 my-5 rounded-lg shadow-2xl">
            <h3 class="font-bold text-xl">コンサルタントID: {{ consultant.consultant_id }}</h3>
            <p class="mt-3 text-xl">相談一回（１時間）の相談料：{{ consultant.fee_per_hour_in_yen }} 円</p>
            <div class="mt-3 text-xl">評価：<span v-if="consultant.rating"> {{ consultant.rating }}</span><span v-else>0</span>/5（評価件数：{{ consultant.num_of_rated }} 件）</div>
            <div class="mt-5 font-bold text-xl">職務経歴概要</div>
            <ul>
              <li v-for="(consultantCareerDescription, index) in consultant.careers" v-bind:key="createUniqueKeyOfConsultantCareerDescription(consultantCareerDescription)">
                <div class="mt-2">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">職務経歴概要{{ index + 1 }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div class="mt-2 justify-self-start col-span-1">勤務先名称</div><div class="justify-self-start col-span-2">{{ consultantCareerDescription.company_name }}</div>
                    <div v-if="consultantCareerDescription.profession" class="mt-2 justify-self-start col-span-1">職種</div><div v-if="consultantCareerDescription.profession" class="justify-self-start col-span-2">{{ consultantCareerDescription.profession }}</div>
                    <div v-if="consultantCareerDescription.office" class="mt-2 justify-self-start col-span-1">勤務地</div><div v-if="consultantCareerDescription.office" class="justify-self-start col-span-2">{{ consultantCareerDescription.office }}</div>
                  </div>
                </div>
              </li>
            </ul>
            <div data-test="consultant-detail-link" class="grid-cols-3 flex justify-end">
              <router-link v-bind:to="{ name: 'ConsultantDetailPage', params: { consultant_id: consultant.consultant_id } }" target="_blank" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">詳細を確認する</router-link>
            </div>
          </div>
        </div>
        <div data-test="page-move-buttons" v-if="pages.length > 1" class="w-fit mb-4 bg-white px-4 py-3 rounded-lg text-black text-xl flex self-end">
          <button data-test="to-first-button" v-on:click="getConsultantsByPageIndex(firstPage)" v-if="currentPage > firstPage" class="bg-gray-600 hover:bg-gray-700 text-white font-bold m-2 px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none" >&lt;&lt;</button>
          <button data-test="to-prev-button" v-on:click="getConsultantsByPageIndex(currentPage - 1)" v-if="currentPage > firstPage" class="bg-gray-600 hover:bg-gray-700 text-white font-bold m-2 px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none" >&lt;</button>
          <div v-bind:data-test="'page-index-' + page" v-for="page in pages" v-bind:key="page">
            <button v-if="page === currentPage" v-on:click="getConsultantsByPageIndex(page)" class="bg-gray-400 hover:bg-gray-500 text-white font-bold m-2 px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">{{ page + 1 }}</button>
            <button v-else v-on:click="getConsultantsByPageIndex(page)" class="bg-gray-600 hover:bg-gray-700 text-white font-bold m-2 px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">{{ page + 1 }}</button>
          </div>
          <button data-test="to-next-button" v-on:click="getConsultantsByPageIndex(currentPage + 1)" v-if="currentPage < lastPage" class="bg-gray-600 hover:bg-gray-700 text-white font-bold m-2 px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none" >&gt;</button>
          <button data-test="to-last-button" v-on:click="getConsultantsByPageIndex(lastPage)" v-if="currentPage < lastPage" class="bg-gray-600 hover:bg-gray-700 text-white font-bold m-2 px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none" >&gt;&gt;</button>
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
import { ConsultantCareerDescription, ConsultantsSearchResult } from '@/util/personalized/consultant-list/ConsultantsSearchResult'

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
      const total = Math.max(0, consultantsSearchResult.value.total - 1)
      return Math.floor(total / searchParam.value.size)
    })
    const pages = computed(() => {
      const pageSize = 2
      if (!searchParam.value) {
        return []
      }
      const min = Math.max(0, currentPage.value - pageSize)
      const max = Math.min(lastPage.value, currentPage.value + pageSize) + 1
      const pages = []
      for (let i = min; i < max; i++) {
        pages.push(i)
      }
      return pages
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

    const getConsultantsByPageIndex = async (page: number) => {
      if (!searchParam.value) {
        error.exists = true
        error.message = Message.NO_CONSULTANT_SEARCH_PARAM_FOUND_MESSAGE
        return
      }
      const from = page * searchParam.value.size
      searchParam.value.from = from
      searchConsultants(searchParam.value)
    }

    const createUniqueKeyOfConsultantCareerDescription = (consultantCareerDescription: ConsultantCareerDescription): string => {
      const companyName = consultantCareerDescription.company_name
      const profession = consultantCareerDescription.profession === null ? 'null' : consultantCareerDescription.profession
      const office = consultantCareerDescription.office === null ? 'null' : consultantCareerDescription.office
      // 半角記号を禁止していてメンバー内に , を含むものがないため、区切り文字として使用して問題なし
      return [companyName, profession, office].join(',')
    }

    return {
      postConsultantsSearchDone,
      error,
      consultantsSearchResult,
      searchParam,
      currentPage,
      firstPage,
      lastPage,
      pages,
      sortParam,
      onSortParamChanged,
      getConsultantsByPageIndex,
      createUniqueKeyOfConsultantCareerDescription
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
