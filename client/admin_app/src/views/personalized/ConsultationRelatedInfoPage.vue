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
            出金待ち情報は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="awaitingWithdrawalErrMessage"/>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">領収書情報</h3>
        <div v-if="!receiptOfConsultationErrMessage">
          <div v-if="receiptOfConsultation" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">ユーザーアカウントID</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.user_account_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">コンサルタントID</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.consultant_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談日時</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.meeting_at }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談料（円）</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.fee_per_hour_in_yen }}</div>
            <div class="mt-2 justify-self-start col-span-3">プラットフォーム手数料率（％）</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.platform_fee_rate_in_percentage }}</div>
            <div class="mt-2 justify-self-start col-span-3">振込手数料（円）</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.transfer_fee_in_yen }}</div>
            <div class="mt-2 justify-self-start col-span-3">報酬</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.reward }}</div>
            <div class="mt-2 justify-self-start col-span-3">入金者</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.sender_name }}</div>
            <div class="mt-2 justify-self-start col-span-3">銀行コード</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.bank_code }}</div>
            <div class="mt-2 justify-self-start col-span-3">支店コード</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.branch_code }}</div>
            <div class="mt-2 justify-self-start col-span-3">口座種別</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.account_type }}</div>
            <div class="mt-2 justify-self-start col-span-3">口座番号</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.account_number }}</div>
            <div class="mt-2 justify-self-start col-span-3">口座名義人</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.account_holder_name }}</div>
            <div class="mt-2 justify-self-start col-span-3">出金確認者</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.withdrawal_confirmed_by }}</div>
            <div class="mt-2 justify-self-start col-span-3">出金確認日時</div><div class="mt-2 justify-self-start col-span-4">{{ receiptOfConsultation.created_at }}</div>
          </div>
          <div v-else class="m-4 text-2xl">
            領収書情報は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="receiptOfConsultationErrMessage"/>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">返金情報</h3>
        <div v-if="!refundedPaymentErrMessage">
          <div v-if="refundedPayment" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ refundedPayment.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">ユーザーアカウントID</div><div class="mt-2 justify-self-start col-span-4">{{ refundedPayment.user_account_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">コンサルタントID</div><div class="mt-2 justify-self-start col-span-4">{{ refundedPayment.consultant_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談日時</div><div class="mt-2 justify-self-start col-span-4">{{ refundedPayment.meeting_at }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談料（円）</div><div class="mt-2 justify-self-start col-span-4">{{ refundedPayment.fee_per_hour_in_yen }}</div>
            <div class="mt-2 justify-self-start col-span-3">振込手数料（円）</div><div class="mt-2 justify-self-start col-span-4">{{ refundedPayment.transfer_fee_in_yen }}</div>
            <div class="mt-2 justify-self-start col-span-3">入金者</div><div class="mt-2 justify-self-start col-span-4">{{ refundedPayment.sender_name }}</div>
            <div class="mt-2 justify-self-start col-span-3">返金理由</div><div class="mt-2 justify-self-start col-span-4">{{ refundedPayment.reason }}</div>
            <div class="mt-2 justify-self-start col-span-3">返金確認者</div><div class="mt-2 justify-self-start col-span-4">{{ refundedPayment.refund_confirmed_by }}</div>
            <div class="mt-2 justify-self-start col-span-3">返金確認日時</div><div class="mt-2 justify-self-start col-span-4">{{ refundedPayment.created_at }}</div>
          </div>
          <div v-else class="m-4 text-2xl">
            返金情報は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="refundedPaymentErrMessage"/>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">放置された報酬情報</h3>
        <div v-if="!leftAwaitingWithdrawalErrMessage">
          <div v-if="leftAwaitingWithdrawal" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ leftAwaitingWithdrawal.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">ユーザーアカウントID</div><div class="mt-2 justify-self-start col-span-4">{{ leftAwaitingWithdrawal.user_account_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">コンサルタントID</div><div class="mt-2 justify-self-start col-span-4">{{ leftAwaitingWithdrawal.consultant_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談日時</div><div class="mt-2 justify-self-start col-span-4">{{ leftAwaitingWithdrawal.meeting_at }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談料（円）</div><div class="mt-2 justify-self-start col-span-4">{{ leftAwaitingWithdrawal.fee_per_hour_in_yen }}</div>
            <div class="mt-2 justify-self-start col-span-3">入金者</div><div class="mt-2 justify-self-start col-span-4">{{ leftAwaitingWithdrawal.sender_name }}</div>
            <div class="mt-2 justify-self-start col-span-3">確認者</div><div class="mt-2 justify-self-start col-span-4">{{ leftAwaitingWithdrawal.confirmed_by }}</div>
            <div class="mt-2 justify-self-start col-span-3">確認日時</div><div class="mt-2 justify-self-start col-span-4">{{ leftAwaitingWithdrawal.created_at }}</div>
          </div>
          <div v-else class="m-4 text-2xl">
            放置された報酬情報は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="leftAwaitingWithdrawalErrMessage"/>
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
import { AwaitingPayment } from '@/util/personalized/consultation/awaiting-payment/AwaitingPayment'
import { useGetAwaitingPaymentByConsultationId } from '@/util/personalized/consultation/awaiting-payment/useGetAwaitingPaymentByConsultationId'
import { GetAwaitingPaymentByConsultationIdResp } from '@/util/personalized/consultation/awaiting-payment/GetAwaitingPaymentByConsultationIdResp'
import { useGetNeglectedPaymentByConsultationId } from '@/util/personalized/consultation/neglected-payment/useGetNeglectedPaymentByConsultationId'
import { GetNeglectedPaymentByConsultationIdResp } from '@/util/personalized/consultation/neglected-payment/GetNeglectedPaymentByConsultationIdResp'
import { NeglectedPayment } from '@/util/personalized/NeglectedPayment'
import { AwaitingWithdrawal } from '@/util/personalized/consultation/awaiting-withdrawal/AwaitingWithdrawal'
import { GetAwaitingWithdrawalByConsultationIdResp } from '@/util/personalized/consultation/awaiting-withdrawal/GetAwaitingWithdrawalByConsultationIdResp'
import { useGetAwaitingWithdrawalByConsultationId } from '@/util/personalized/consultation/awaiting-withdrawal/useGetAwaitingWithdrawalByConsultationId'
import { ReceiptOfConsultation } from '@/util/personalized/ReceiptOfConsultation'
import { useGetReceiptOfConsultationByConsultationId } from '@/util/personalized/consultation/receipt-of-consultation/useGetReceiptOfConsultationByConsultationId'
import { GetReceiptOfConsultationByConsultationIdResp } from '@/util/personalized/consultation/receipt-of-consultation/GetReceiptOfConsultationByConsultationIdResp'
import { RefundedPayment } from '@/util/personalized/RefundedPayment'
import { useGetRefundedPaymentByConsultationId } from '@/util/personalized/consultation/refunded-payment/useGetRefundedPaymentByConsultationId'
import { GetRefundedPaymentByConsultationIdResp } from '@/util/personalized/consultation/refunded-payment/GetRefundedPaymentByConsultationIdResp'
import { LeftAwaitingWithdrawal } from '@/util/personalized/LeftAwaitingWithdrawal'
import { useGetLeftAwaitingWithdrawalByConsultationId } from '@/util/personalized/consultation/left-awaiting-withdrawal/useGetLeftAwaitingWithdrawalByConsultationId'
import { GetLeftAwaitingWithdrawalByConsultationIdResp } from '@/util/personalized/consultation/left-awaiting-withdrawal/GetLeftAwaitingWithdrawalByConsultationIdResp'

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

    const receiptOfConsultation = ref(null as ReceiptOfConsultation | null)
    const receiptOfConsultationErrMessage = ref(null as string | null)

    const {
      getReceiptOfConsultationByConsultationIdDone,
      getReceiptOfConsultationByConsultationIdFunc
    } = useGetReceiptOfConsultationByConsultationId()

    const findReceiptOfConsultation = async () => {
      try {
        const response = await getReceiptOfConsultationByConsultationIdFunc(consultationId)
        if (!(response instanceof GetReceiptOfConsultationByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          receiptOfConsultationErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        receiptOfConsultation.value = response.getReceiptOfConsultation()
      } catch (e) {
        receiptOfConsultationErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const refundedPayment = ref(null as RefundedPayment | null)
    const refundedPaymentErrMessage = ref(null as string | null)

    const {
      getRefundedPaymentByConsultationIdDone,
      getRefundedPaymentByConsultationIdFunc
    } = useGetRefundedPaymentByConsultationId()

    const findRefundedPayment = async () => {
      try {
        const response = await getRefundedPaymentByConsultationIdFunc(consultationId)
        if (!(response instanceof GetRefundedPaymentByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          refundedPaymentErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        refundedPayment.value = response.getRefundedPayment()
      } catch (e) {
        refundedPaymentErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const leftAwaitingWithdrawal = ref(null as LeftAwaitingWithdrawal | null)
    const leftAwaitingWithdrawalErrMessage = ref(null as string | null)

    const {
      getLeftAwaitingWithdrawalByConsultationIdDone,
      getLeftAwaitingWithdrawalByConsultationIdFunc
    } = useGetLeftAwaitingWithdrawalByConsultationId()

    const findLeftAwaitingWithdrawal = async () => {
      try {
        const response = await getLeftAwaitingWithdrawalByConsultationIdFunc(consultationId)
        if (!(response instanceof GetLeftAwaitingWithdrawalByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          leftAwaitingWithdrawalErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        leftAwaitingWithdrawal.value = response.getLeftAwaitingWithdrawal()
      } catch (e) {
        leftAwaitingWithdrawalErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    onMounted(async () => {
      await findConsultation()
      await findUserRating()
      await findConsultantRating()
      await findAwaitingPayment()
      await findNeglectedPayment()
      await findAwaitingWithdrawal()
      await findReceiptOfConsultation()
      await findRefundedPayment()
      await findLeftAwaitingWithdrawal()
    })

    const requestsDone = computed(() => {
      return getConsultationByConsultationIdDone.value &&
              getUserRatingByConsultationIdDone.value &&
              getConsultantRatingByConsultationIdDone.value &&
              getAwaitingPaymentByConsultationIdDone.value &&
              getNeglectedPaymentByConsultationIdDone.value &&
              getAwaitingWithdrawalByConsultationIdDone.value &&
              getReceiptOfConsultationByConsultationIdDone.value &&
              getRefundedPaymentByConsultationIdDone.value &&
              getLeftAwaitingWithdrawalByConsultationIdDone.value
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
      receiptOfConsultation,
      receiptOfConsultationErrMessage,
      refundedPayment,
      refundedPaymentErrMessage,
      leftAwaitingWithdrawal,
      leftAwaitingWithdrawalErrMessage
    }
  }
})
</script>
