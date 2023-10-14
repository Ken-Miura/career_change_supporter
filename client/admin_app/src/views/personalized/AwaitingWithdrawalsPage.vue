<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getAwaitingWithdrawalsDone || !postLeftAwaitingWithdrawalDone || !postReceiptOfConsultationDone || !postRefundFromAwaitingWithdrawalDone" class="m-6">
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
          <h3 class="font-bold text-xl lg:text-2xl">出金待ちリスト</h3>
            <ul>
              <li v-for="item in items" v-bind:key="item.consultation_id">
                <div class="mt-6">
                  <div class="text-lg lg:text-xl bg-gray-600 text-white font-bold rounded-t px-4 py-2">相談ID{{ item.consultation_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-lg lg:text-xl grid grid-cols-3">
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">ユーザーID</div><div class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.user_account_id }}</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">コンサルタントID</div><div class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.consultant_id }}</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">相談日時</div><div class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.meeting_at }}</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">相談料（円）</div><div class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.fee_per_hour_in_yen }}</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">入金者</div><div v-if="item.sender_name" class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.sender_name }}</div><div v-else class="my-1 lg:my-2 justify-self-start col-span-2">想定しない値です。管理者に連絡して下さい</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">入金確認者</div><div class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.payment_confirmed_by }}</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">入金確認日時</div><div class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.created_at }}</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">銀行コード</div><div v-if="item.bank_code" class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.bank_code }}</div><div v-else class="my-1 lg:my-2 justify-self-start col-span-2">既に口座情報が削除されています</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">支店コード</div><div v-if="item.branch_code" class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.branch_code }}</div><div v-else class="my-1 lg:my-2 justify-self-start col-span-2">既に口座情報が削除されています</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">口座種別</div><div v-if="item.account_type" class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.account_type }}</div><div v-else class="my-1 lg:my-2 justify-self-start col-span-2">既に口座情報が削除されています</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">口座番号</div><div v-if="item.account_number" class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.account_number }}</div><div v-else class="my-1 lg:my-2 justify-self-start col-span-2">既に口座情報が削除されています</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">口座名義人</div><div v-if="item.account_holder_name" class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.account_holder_name }}</div><div v-else class="my-1 lg:my-2 justify-self-start col-span-2">既に口座情報が削除されています</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">プラットフォーム手数料率（％）</div><div class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.platform_fee_rate_in_percentage }}</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1">振込手数料（円）</div><div class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.transfer_fee_in_yen }}</div>
                  <div class="my-1 lg:my-2 justify-self-start col-span-1"><span class=" text-red-500">報酬（円）<br>（振り込む金額）</span></div><div class="my-1 lg:my-2 justify-self-start col-span-2">{{ item.reward }}</div>
                  <button v-on:click="confirmWithdrawal(item.consultation_id)" class="mt-6 col-span-3 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">報酬を渡したので領収書へ移動</button>
                  <button v-on:click="confirmLeftAwaitingWithdrawal(item.consultation_id)" class="mt-6 col-span-3 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">既に口座情報が削除されているので放置された報酬へ移動</button>
                  <button v-on:click="confirmRefund(item.consultation_id)" class="mt-6 col-span-3 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">返金を行ったので返金済みへ移動</button>
                </div>
              </div>
            </li>
          </ul>
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
import { computed, defineComponent, onMounted, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { Message } from '@/util/Message'
import { useGetAwaitingWithdrawals } from '@/util/personalized/awaiting-withdrawal/useGetAwaitingWithdrawals'
import { AwaitingWithdrawal } from '@/util/personalized/awaiting-withdrawal/AwaitingWithdrawal'
import { GetAwaitingWithdrawalsResp } from '@/util/personalized/awaiting-withdrawal/GetAwaitingWithdrawalsResp'
import { usePostLeftAwaitingWithdrawal } from '@/util/personalized/awaiting-withdrawal/usePostLeftAwaitingWithdrawal'
import { PostLeftAwaitingWithdrawalResp } from '@/util/personalized/awaiting-withdrawal/PostLeftAwaitingWithdrawalResp'
import { usePostReceiptOfConsultation } from '@/util/personalized/awaiting-withdrawal/usePostReceiptOfConsultation'
import { PostReceiptOfConsultationResp } from '@/util/personalized/awaiting-withdrawal/PostReceiptOfConsultationResp'
import { usePostRefundFromAwaitingWithdrawal } from '@/util/personalized/awaiting-withdrawal/usePostRefundFromAwaitingWithdrawal'
import { PostRefundFromAwaitingWithdrawalResp } from '@/util/personalized/awaiting-withdrawal/PostRefundFromAwaitingWithdrawalResp'

export default defineComponent({
  name: 'AwaitingWithdrawalsPage',
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
      getAwaitingWithdrawalsDone,
      getAwaitingWithdrawalsFunc
    } = useGetAwaitingWithdrawals()

    const router = useRouter()
    const route = useRoute()
    const query = route.query
    const page = ref(parseInt(query.page as string))
    const perPage = ref(parseInt(query['per-page'] as string))

    const items = ref([] as AwaitingWithdrawal[])
    const prevDisabled = computed(() => page.value <= 0)
    const nextDisabled = computed(() => items.value.length < perPage.value)
    const prev = async () => {
      await router.push(`/awaiting-withdrawals?page=${(page.value - 1)}&per-page=${perPage.value}`)
    }
    const next = async () => {
      await router.push(`/awaiting-withdrawals?page=${(page.value + 1)}&per-page=${perPage.value}`)
    }

    const getItems = async (page: number, perPage: number) => {
      try {
        const response = await getAwaitingWithdrawalsFunc(page, perPage)
        if (!(response instanceof GetAwaitingWithdrawalsResp)) {
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
    }

    watch(route, async (newRoute) => {
      const query = newRoute.query
      page.value = parseInt(query.page as string)
      perPage.value = parseInt(query['per-page'] as string)
      await getItems(page.value, perPage.value)
    })

    onMounted(async () => {
      await getItems(page.value, perPage.value)
    })

    const {
      postReceiptOfConsultationDone,
      postReceiptOfConsultationFunc
    } = usePostReceiptOfConsultation()

    const confirmWithdrawal = async (consultationId: number) => {
      try {
        const response = await postReceiptOfConsultationFunc(consultationId)
        if (!(response instanceof PostReceiptOfConsultationResp)) {
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
        await getItems(page.value, perPage.value)
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const {
      postLeftAwaitingWithdrawalDone,
      postLeftAwaitingWithdrawalFunc
    } = usePostLeftAwaitingWithdrawal()

    const confirmLeftAwaitingWithdrawal = async (consultationId: number) => {
      try {
        const response = await postLeftAwaitingWithdrawalFunc(consultationId)
        if (!(response instanceof PostLeftAwaitingWithdrawalResp)) {
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
        await getItems(page.value, perPage.value)
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const {
      postRefundFromAwaitingWithdrawalDone,
      postRefundFromAwaitingWithdrawalFunc
    } =
    usePostRefundFromAwaitingWithdrawal()

    const confirmRefund = async (consultationId: number) => {
      try {
        const response = await postRefundFromAwaitingWithdrawalFunc(consultationId)
        if (!(response instanceof PostRefundFromAwaitingWithdrawalResp)) {
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
        await getItems(page.value, perPage.value)
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    return {
      error,
      getAwaitingWithdrawalsDone,
      postLeftAwaitingWithdrawalDone,
      postReceiptOfConsultationDone,
      postRefundFromAwaitingWithdrawalDone,
      items,
      confirmWithdrawal,
      confirmLeftAwaitingWithdrawal,
      confirmRefund,
      prevDisabled,
      nextDisabled,
      prev,
      next
    }
  }
})
</script>
