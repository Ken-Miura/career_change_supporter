<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getAwaitingPaymentsDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="error.exists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="error.message"/>
        </div>
      </div>
      <div v-else>
        <div data-test="list" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
          <!-- <div class="mt-4 bg-white px-4 py-3 text-black text-xl grid grid-cols-4">
            <div class="mt-2 justify-self-start col-span-2">依頼時刻</div>
            <div class="mt-2 justify-self-start col-span-1">会社名</div>
            <div class="mt-2 justify-self-start col-span-1"></div>
          </div>
          <ul data-test="items">
            <li v-for="item in items" v-bind:key="item.create_career_req_id">
              <div class="mt-4">
                <div class="border border-gray-600 rounded bg-white px-4 py-3 text-black text-xl grid grid-cols-4">
                  <div class="mt-3 justify-self-start col-span-2">{{ item.requested_at.getFullYear() }}年{{ (item.requested_at.getMonth() + 1).toString().padStart(2, '0') }}月{{ item.requested_at.getDate().toString().padStart(2, '0') }}日{{ item.requested_at.getHours().toString().padStart(2, '0') }}時{{ item.requested_at.getMinutes().toString().padStart(2, '0') }}分{{ item.requested_at.getSeconds().toString().padStart(2, '0') }}秒</div>
                  <div class="mt-3 justify-self-start col-span-1">{{ item.company_name }}</div>
                  <button class="col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200" v-on:click="moveToCreateCareerRequestDetailPage(item.create_career_req_id)">詳細を確認する</button>
                </div>
              </div>
            </li>
          </ul> -->
          <div class="mt-4 bg-white px-4 py-3 text-black text-xl grid grid-cols-2">
            <button data-test="prev-button" v-on:click="prev" v-bind:disabled="prevDisabled" class="col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold m-2 px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none" >＜</button>
            <button data-test="next-button" v-on:click="next" v-bind:disabled="nextDisabled" class="col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold m-2 px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none" >＞</button>
          </div>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex flex-col text-white">
      <router-link to="/admin-menu" class="hover:underline text-center">管理メニューへ</router-link>
      <router-link to="/" class="mt-6 hover:underline text-center">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useGetAwaitingPayments } from '@/util/personalized/awaiting-payment/useGetAwaitingPayments'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { Message } from '@/util/Message'
import { GetAwaitingPaymentsResp } from '@/util/personalized/awaiting-payment/GetAwaitingPaymentsResp'
import { AwaitingPayment } from '@/util/personalized/awaiting-payment/AwaitingPayment'

export default defineComponent({
  name: 'AwaitingPaymentsPage',
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

    const {
      getAwaitingPaymentsDone,
      getAwaitingPaymentsFunc
    } = useGetAwaitingPayments()

    const router = useRouter()
    const route = useRoute()
    const query = route.query
    const page = ref(parseInt(query.page as string))
    const perPage = ref(parseInt(query['per-page'] as string))

    const items = ref([] as AwaitingPayment[])
    const prevDisabled = computed(() => page.value <= 0)
    const nextDisabled = computed(() => items.value.length < perPage.value)
    const prev = async () => {
      await router.push(`/awaiting-payments?page=${(page.value - 1)}&per-page=${perPage.value}`)
    }
    const next = async () => {
      await router.push(`/awaiting-payments?page=${(page.value + 1)}&per-page=${perPage.value}`)
    }

    onMounted(async () => {
      try {
        const response = await getAwaitingPaymentsFunc(page.value, perPage.value)
        if (!(response instanceof GetAwaitingPaymentsResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          error.exists = true
          error.message = createErrorMessage(response.getApiError().getCode())
          return
        }
        items.value = response.getItems()
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    return {
      error,
      getAwaitingPaymentsDone,
      items,
      prevDisabled,
      nextDisabled,
      prev,
      next
    }
  }
})
</script>
