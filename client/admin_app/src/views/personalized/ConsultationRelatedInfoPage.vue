<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!requestsDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">相談</h3>
        <div v-if="!consultationErrMessage">
          <div v-if="consultation" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ consultation.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">コンサルタントID</div><div class="mt-2 justify-self-start col-span-4">{{ consultation.consultant_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">ユーザーアカウントID</div><div class="mt-2 justify-self-start col-span-4">{{ consultation.user_account_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談日時</div><div class="mt-2 justify-self-start col-span-4">{{ consultation.meeting_at }}</div>
            <div class="mt-2 justify-self-start col-span-3">部屋名</div><div class="mt-2 justify-self-start col-span-4">{{ consultation.room_name }}</div>
            <div class="mt-2 justify-self-start col-span-3">ユーザー入室日時</div><div v-if="consultation.user_account_entered_at" class="mt-2 justify-self-start col-span-4">{{ consultation.user_account_entered_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">入室記録なし</div>
            <div class="mt-2 justify-self-start col-span-3">コンサルタント入室日時</div><div v-if="consultation.consultant_entered_at" class="mt-2 justify-self-start col-span-4">{{ consultation.consultant_entered_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">入室記録なし</div>
          </div>
          <div v-else class="m-4 text-2xl">
            相談は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="consultationErrMessage"/>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">コンサルタントからのユーザーに対する評価</h3>
        <div v-if="!userRatingErrMessage">
          <div v-if="userRating" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ userRating.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">評価</div><div v-if="userRating.rating" class="mt-2 justify-self-start col-span-4">{{ userRating.rating }}</div><div v-else class="mt-2 justify-self-start col-span-4">未評価</div>
            <div class="mt-2 justify-self-start col-span-3">評価日時</div><div v-if="userRating.rated_at" class="mt-2 justify-self-start col-span-4">{{ userRating.rated_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">未評価</div>
          </div>
          <div v-else class="m-4 text-2xl">
            コンサルタントからのユーザーに対する評価は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="userRatingErrMessage"/>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">ユーザーからのコンサルタントに対する評価</h3>
        <div v-if="!consultantRatingErrMessage">
          <div v-if="consultantRating" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ consultantRating.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">評価</div><div v-if="consultantRating.rating" class="mt-2 justify-self-start col-span-4">{{ consultantRating.rating }}</div><div v-else class="mt-2 justify-self-start col-span-4">未評価</div>
            <div class="mt-2 justify-self-start col-span-3">評価日時</div><div v-if="consultantRating.rated_at" class="mt-2 justify-self-start col-span-4">{{ consultantRating.rated_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">未評価</div>
          </div>
          <div v-else class="m-4 text-2xl">
            ユーザーからのコンサルタントに対する評価は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="consultantRatingErrMessage"/>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">入金待ち情報</h3>
        <div v-if="!awaitingPaymentErrMessage">
          <div v-if="awaitingPayment" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingPayment.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">ユーザーアカウントID</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingPayment.user_account_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">コンサルタントID</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingPayment.consultant_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談料（円）</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingPayment.fee_per_hour_in_yen }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談日時</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingPayment.meeting_at }}</div>
            <div class="mt-2 justify-self-start col-span-3">作成日時<br>（相談受け付け承認日時）</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingPayment.created_at }}</div>
          </div>
          <div v-else class="m-4 text-2xl">
            入金待ち情報は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="awaitingPaymentErrMessage"/>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">無視された入金情報</h3>
        <div v-if="!neglectedPaymentErrMessage">
          <div v-if="neglectedPayment" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ neglectedPayment.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">ユーザーアカウントID</div><div class="mt-2 justify-self-start col-span-4">{{ neglectedPayment.user_account_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">コンサルタントID</div><div class="mt-2 justify-self-start col-span-4">{{ neglectedPayment.consultant_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談料（円）</div><div class="mt-2 justify-self-start col-span-4">{{ neglectedPayment.fee_per_hour_in_yen }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談日時</div><div class="mt-2 justify-self-start col-span-4">{{ neglectedPayment.meeting_at }}</div>
            <div class="mt-2 justify-self-start col-span-3">確認者</div><div class="mt-2 justify-self-start col-span-4">{{ neglectedPayment.neglect_confirmed_by }}</div>
            <div class="mt-2 justify-self-start col-span-3">確認日時</div><div class="mt-2 justify-self-start col-span-4">{{ neglectedPayment.created_at }}</div>
          </div>
          <div v-else class="m-4 text-2xl">
            無視された入金情報は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="neglectedPaymentErrMessage"/>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">出金待ち情報</h3>
        <div v-if="!awaitingWithdrawalErrMessage">
          <div v-if="awaitingWithdrawal" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingWithdrawal.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">ユーザーアカウントID</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingWithdrawal.user_account_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">コンサルタントID</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingWithdrawal.consultant_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談日時</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingWithdrawal.meeting_at }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談料（円）</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingWithdrawal.fee_per_hour_in_yen }}</div>
            <div class="mt-2 justify-self-start col-span-3">入金者</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingWithdrawal.sender_name }}</div>
            <div class="mt-2 justify-self-start col-span-3">入金確認者</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingWithdrawal.payment_confirmed_by }}</div>
            <div class="mt-2 justify-self-start col-span-3">入金確認日時</div><div class="mt-2 justify-self-start col-span-4">{{ awaitingWithdrawal.created_at }}</div>
          </div>
          <div v-else class="m-4 text-2xl">
            無視された入金情報は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="awaitingWithdrawalErrMessage"/>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">領収書情報</h3>
        <div v-if="!receiptErrMessage">
          <div v-if="receipt" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">領収書番号</div><div class="mt-2 justify-self-start col-span-4">{{ receipt.receipt_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ receipt.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">チャージID</div><div class="mt-2 justify-self-start col-span-4">{{ receipt.charge_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談料（円）</div><div class="mt-2 justify-self-start col-span-4">{{ receipt.fee_per_hour_in_yen }}</div>
            <div class="mt-2 justify-self-start col-span-3">プラットフォーム利用手数料割合（%）</div><div class="mt-2 justify-self-start col-span-4">{{ receipt.platform_fee_rate_in_percentage }}</div>
            <div class="mt-2 justify-self-start col-span-3">支払い確定日時</div><div class="mt-2 justify-self-start col-span-4">{{ receipt.settled_at }}</div>
            <div class="mt-4 col-span-7">
              <div class="text-2xl justify-self-start col-span-6 pt-3 font-bold">
                <p>返金</p>
              </div>
              <div class="mt-2 ml-2 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
                <div class="p-4 text-xl grid grid-cols-6 justify-center items-center">
                  <div class="col-span-5">返金が適正であることを確認しました</div>
                  <input v-model="refundReqConfirmation" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
                </div>
              </div>
              <div>
                <button v-on:click="refundReq" v-bind:disabled="!refundReqConfirmation" class="mt-4 ml-2 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">返金する</button>
              </div>
              <div v-if="refundErrMessage" class="mt-4">
                <AlertMessage v-bind:message="refundErrMessage"/>
              </div>
            </div>
          </div>
          <div v-else class="m-4 text-2xl">
            領収書情報は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="receiptErrMessage"/>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">返金情報</h3>
        <div v-if="!refundErrMessage">
          <div v-if="refund" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">返金番号</div><div class="mt-2 justify-self-start col-span-4">{{ refund.refund_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ refund.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">チャージID</div><div class="mt-2 justify-self-start col-span-4">{{ refund.charge_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談料（円）</div><div class="mt-2 justify-self-start col-span-4">{{ refund.fee_per_hour_in_yen }}</div>
            <div class="mt-2 justify-self-start col-span-3">プラットフォーム利用手数料割合（%）</div><div class="mt-2 justify-self-start col-span-4">{{ refund.platform_fee_rate_in_percentage }}</div>
            <div class="mt-2 justify-self-start col-span-3">支払い確定日時</div><div class="mt-2 justify-self-start col-span-4">{{ refund.settled_at }}</div>
            <div class="mt-2 justify-self-start col-span-3">返金日時</div><div class="mt-2 justify-self-start col-span-4">{{ refund.refunded_at }}</div>
          </div>
          <div v-else class="m-4 text-2xl">
            返金情報は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="refundErrMessage"/>
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
import { defineComponent, ref, onMounted, computed } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRoute, useRouter } from 'vue-router'
import { Consultation } from '@/util/personalized/Consultation'
import { useGetConsultationByConsultationId } from '@/util/personalized/consultation/useGetConsultationByConsultationId'
import { Message } from '@/util/Message'
import { GetConsultationByConsultationIdResp } from '@/util/personalized/consultation/GetConsultationByConsultationIdResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { useGetUserRatingByConsultationId } from '@/util/personalized/consultation/user-rating/useGetUserRatingByConsultationId'
import { UserRating } from '@/util/personalized/consultation/user-rating/UserRating'
import { GetUserRatingByConsultationIdResp } from '@/util/personalized/consultation/user-rating/GetUserRatingByConsultationIdResp'
import { useGetConsultantRatingByConsultationId } from '@/util/personalized/consultation/consultant-rating/useGetConsultantRatingByConsultationId'
import { GetConsultantRatingByConsultationIdResp } from '@/util/personalized/consultation/consultant-rating/GetConsultantRatingByConsultationIdResp'
import { ConsultantRating } from '@/util/personalized/consultation/consultant-rating/ConsultantRating'
import { Receipt } from '@/util/personalized/consultation/receipt/Receipt'
import { useReceiptByConsultationId } from '@/util/personalized/consultation/receipt/useGetReceiptByConsultationId'
import { GetReceiptByConsultationIdResp } from '@/util/personalized/consultation/receipt/GetReceiptByConsultationIdResp'
import { Refund } from '@/util/personalized/consultation/refund/Refund'
import { useGetRefundByConsultationId } from '@/util/personalized/consultation/refund/useGetRefundByConsultationId'
import { GetRefundByConsultationIdResp } from '@/util/personalized/consultation/refund/GetRefundByConsultationIdResp'
import { usePostRefundReq } from '@/util/personalized/consultation/refund_req/usePostRefundReq'
import { PostRefundReqResp } from '@/util/personalized/consultation/refund_req/PostRefundReqResp'
import { AwaitingPayment } from '@/util/personalized/consultation/awaiting-payment/AwaitingPayment'
import { useGetAwaitingPaymentByConsultationId } from '@/util/personalized/consultation/awaiting-payment/useGetAwaitingPaymentByConsultationId'
import { GetAwaitingPaymentByConsultationIdResp } from '@/util/personalized/consultation/awaiting-payment/GetAwaitingPaymentByConsultationIdResp'
import { useGetNeglectedPaymentByConsultationId } from '@/util/personalized/consultation/neglected-payment/useGetNeglectedPaymentByConsultationId'
import { GetNeglectedPaymentByConsultationIdResp } from '@/util/personalized/consultation/neglected-payment/GetNeglectedPaymentByConsultationIdResp'
import { NeglectedPayment } from '@/util/personalized/NeglectedPayment'
import { AwaitingWithdrawal } from '@/util/personalized/consultation/awaiting-withdrawal/AwaitingWithdrawal'
import { GetAwaitingWithdrawalByConsultationIdResp } from '@/util/personalized/consultation/awaiting-withdrawal/GetAwaitingWithdrawalByConsultationIdResp'
import { useGetAwaitingWithdrawalByConsultationId } from '@/util/personalized/consultation/awaiting-withdrawal/useGetAwaitingWithdrawalByConsultationId'

export default defineComponent({
  name: 'ConsultationRelatedInfoPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()
    const route = useRoute()
    const consultationId = route.params.consultation_id as string

    const consultation = ref(null as Consultation | null)
    const consultationErrMessage = ref(null as string | null)

    const {
      getConsultationByConsultationIdDone,
      getConsultationByConsultationIdFunc
    } = useGetConsultationByConsultationId()

    const findConsultation = async () => {
      try {
        const response = await getConsultationByConsultationIdFunc(consultationId)
        if (!(response instanceof GetConsultationByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          consultationErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getConsultationResult()
        consultation.value = result.consultation
      } catch (e) {
        consultationErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const userRating = ref(null as UserRating | null)
    const userRatingErrMessage = ref(null as string | null)

    const {
      getUserRatingByConsultationIdDone,
      getUserRatingByConsultationIdFunc
    } = useGetUserRatingByConsultationId()

    const findUserRating = async () => {
      try {
        const response = await getUserRatingByConsultationIdFunc(consultationId)
        if (!(response instanceof GetUserRatingByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          userRatingErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getUserRatingResult()
        userRating.value = result.user_rating
      } catch (e) {
        userRatingErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const consultantRating = ref(null as ConsultantRating | null)
    const consultantRatingErrMessage = ref(null as string | null)

    const {
      getConsultantRatingByConsultationIdDone,
      getConsultantRatingByConsultationIdFunc
    } = useGetConsultantRatingByConsultationId()

    const findConsultantRating = async () => {
      try {
        const response = await getConsultantRatingByConsultationIdFunc(consultationId)
        if (!(response instanceof GetConsultantRatingByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          consultantRatingErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getConsultantRatingResult()
        consultantRating.value = result.consultant_rating
      } catch (e) {
        consultantRatingErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const awaitingPayment = ref(null as AwaitingPayment | null)
    const awaitingPaymentErrMessage = ref(null as string | null)

    const {
      getAwaitingPaymentByConsultationIdDone,
      getAwaitingPaymentByConsultationIdFunc
    } = useGetAwaitingPaymentByConsultationId()

    const findAwaitingPayment = async () => {
      try {
        const response = await getAwaitingPaymentByConsultationIdFunc(consultationId)
        if (!(response instanceof GetAwaitingPaymentByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          awaitingPaymentErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        awaitingPayment.value = response.getAwaitingPayment()
      } catch (e) {
        awaitingPaymentErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const neglectedPayment = ref(null as NeglectedPayment | null)
    const neglectedPaymentErrMessage = ref(null as string | null)

    const {
      getNeglectedPaymentByConsultationIdDone,
      getNeglectedPaymentByConsultationIdFunc
    } = useGetNeglectedPaymentByConsultationId()

    const findNeglectedPayment = async () => {
      try {
        const response = await getNeglectedPaymentByConsultationIdFunc(consultationId)
        if (!(response instanceof GetNeglectedPaymentByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          neglectedPaymentErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        neglectedPayment.value = response.getNeglectedPayment()
      } catch (e) {
        neglectedPaymentErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const awaitingWithdrawal = ref(null as AwaitingWithdrawal | null)
    const awaitingWithdrawalErrMessage = ref(null as string | null)

    const {
      getAwaitingWithdrawalByConsultationIdDone,
      getAwaitingWithdrawalByConsultationIdFunc
    } = useGetAwaitingWithdrawalByConsultationId()

    const findAwaitingWithdrawal = async () => {
      try {
        const response = await getAwaitingWithdrawalByConsultationIdFunc(consultationId)
        if (!(response instanceof GetAwaitingWithdrawalByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          awaitingWithdrawalErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        awaitingWithdrawal.value = response.getAwaitingWithdrawal()
      } catch (e) {
        awaitingWithdrawalErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const receipt = ref(null as Receipt | null)
    const receiptErrMessage = ref(null as string | null)

    const {
      getReceiptByConsultationIdDone,
      getReceiptByConsultationIdFunc
    } = useReceiptByConsultationId()

    const findReceipt = async () => {
      try {
        const response = await getReceiptByConsultationIdFunc(consultationId)
        if (!(response instanceof GetReceiptByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          receiptErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getReceiptResult()
        receipt.value = result.receipt
      } catch (e) {
        receiptErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const refund = ref(null as Refund | null)
    const refundErrMessage = ref(null as string | null)

    const {
      getRefundByConsultationIdDone,
      getRefundByConsultationIdFunc
    } = useGetRefundByConsultationId()

    const findRefund = async () => {
      try {
        const response = await getRefundByConsultationIdFunc(consultationId)
        if (!(response instanceof GetRefundByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          refundErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getRefundResult()
        refund.value = result.refund
      } catch (e) {
        refundErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const {
      postRefundReqDone,
      postRefundReqFunc
    } = usePostRefundReq()

    const refundReqConfirmation = ref(false)
    const refundReqErrMessage = ref(null as string | null)

    const refundReq = async () => {
      if (!receipt.value) {
        refundReqErrMessage.value = `${Message.UNEXPECTED_ERR}: receipt.value is null`
        return
      }
      const receiptId = receipt.value.receipt_id
      try {
        const response = await postRefundReqFunc(receiptId)
        if (!(response instanceof PostRefundReqResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          refundReqErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        refundReqErrMessage.value = null
      } catch (e) {
        refundReqErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      } finally {
        refundReqConfirmation.value = false
      }
      await findReceipt()
      await findRefund()
    }

    onMounted(async () => {
      await findConsultation()
      await findUserRating()
      await findConsultantRating()
      await findAwaitingPayment()
      await findNeglectedPayment()
      await findAwaitingWithdrawal()
      await findReceipt()
      await findRefund()
    })

    const requestsDone = computed(() => {
      return getConsultationByConsultationIdDone.value &&
              getUserRatingByConsultationIdDone.value &&
              getConsultantRatingByConsultationIdDone.value &&
              getAwaitingPaymentByConsultationIdDone.value &&
              getNeglectedPaymentByConsultationIdDone.value &&
              getAwaitingWithdrawalByConsultationIdDone.value &&
              getReceiptByConsultationIdDone.value &&
              getRefundByConsultationIdDone.value &&
              postRefundReqDone.value
    })

    return {
      requestsDone,
      consultation,
      consultationErrMessage,
      userRating,
      userRatingErrMessage,
      consultantRating,
      consultantRatingErrMessage,
      awaitingPayment,
      awaitingPaymentErrMessage,
      neglectedPayment,
      neglectedPaymentErrMessage,
      awaitingWithdrawal,
      awaitingWithdrawalErrMessage,
      receipt,
      receiptErrMessage,
      refund,
      refundErrMessage,
      refundReqConfirmation,
      refundReq,
      refundReqErrMessage
    }
  }
})
</script>
