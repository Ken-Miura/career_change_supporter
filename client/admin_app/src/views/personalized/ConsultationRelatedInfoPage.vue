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
        <h3 class="font-bold text-2xl">決済情報（確保した与信枠の情報）</h3>
        <div v-if="!settlementErrMessage">
          <div v-if="settlement" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">決済番号</div><div class="mt-2 justify-self-start col-span-4">{{ settlement.settlement_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ settlement.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">チャージID</div><div class="mt-2 justify-self-start col-span-4">{{ settlement.charge_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談料（円）</div><div class="mt-2 justify-self-start col-span-4">{{ settlement.fee_per_hour_in_yen }}</div>
            <div class="mt-2 justify-self-start col-span-3">プラットフォーム利用手数料割合（%）</div><div class="mt-2 justify-self-start col-span-4">{{ settlement.platform_fee_rate_in_percentage }}</div>
            <div class="mt-2 justify-self-start col-span-3">与信枠開放日時</div><div class="mt-2 justify-self-start col-span-4">{{ settlement.credit_facilities_expired_at }}</div>
            <div class="mt-4 col-span-7">
              <div class="text-2xl justify-self-start col-span-6 pt-3 font-bold">
                <p>決済の停止</p>
              </div>
              <div class="mt-2 ml-2 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
                <div class="p-4 text-xl grid grid-cols-6 justify-center items-center">
                  <div class="col-span-5">決済の停止が適正であることを確認しました</div>
                  <input v-model="stopSettlementConfirmation" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
                </div>
              </div>
              <div>
                <button v-on:click="stopSettlement" v-bind:disabled="!stopSettlementConfirmation" class="mt-4 ml-2 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">決済を停止する</button>
              </div>
              <div v-if="stopSettlementErrMessage" class="mt-4">
                <AlertMessage v-bind:message="stopSettlementErrMessage"/>
              </div>
            </div>
            <div class="mt-4 col-span-7">
              <div class="text-2xl justify-self-start col-span-6 pt-3 font-bold">
                <p>支払いの確定</p>
              </div>
              <div class="mt-2 ml-2 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
                <div class="p-4 text-xl grid grid-cols-6 justify-center items-center">
                  <div class="col-span-5">支払いの確定が適正であることを確認しました</div>
                  <input v-model="makePaymentConfirmation" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
                </div>
              </div>
              <div>
                <button v-on:click="makePayment" v-bind:disabled="!makePaymentConfirmation" class="mt-4 ml-2 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">支払いを確定する</button>
              </div>
              <div v-if="makePaymentErrMessage" class="mt-4">
                <AlertMessage v-bind:message="makePaymentErrMessage"/>
              </div>
            </div>
          </div>
          <div v-else class="m-4 text-2xl">
            決済情報（確保した与信枠の情報）は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="settlementErrMessage"/>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">停止中の決済情報</h3>
        <div v-if="!stoppedSettlementErrMessage">
          <div v-if="stoppedSettlement" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">停止中決済番号</div><div class="mt-2 justify-self-start col-span-4">{{ stoppedSettlement.stopped_settlement_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ stoppedSettlement.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">チャージID</div><div class="mt-2 justify-self-start col-span-4">{{ stoppedSettlement.charge_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談料（円）</div><div class="mt-2 justify-self-start col-span-4">{{ stoppedSettlement.fee_per_hour_in_yen }}</div>
            <div class="mt-2 justify-self-start col-span-3">プラットフォーム利用手数料割合（%）</div><div class="mt-2 justify-self-start col-span-4">{{ stoppedSettlement.platform_fee_rate_in_percentage }}</div>
            <div class="mt-2 justify-self-start col-span-3">与信枠開放日時</div><div class="mt-2 justify-self-start col-span-4">{{ stoppedSettlement.credit_facilities_expired_at }}</div>
            <div class="mt-2 justify-self-start col-span-3">停止日時</div><div class="mt-2 justify-self-start col-span-4">{{ stoppedSettlement.stopped_at }}</div>
            <div class="mt-4 col-span-7">
              <div class="text-2xl justify-self-start col-span-6 pt-3 font-bold">
                <p>決済の再開</p>
              </div>
              <div class="mt-2 ml-2 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
                <div class="p-4 text-xl grid grid-cols-6 justify-center items-center">
                  <div class="col-span-5">決済の再開が適正であることを確認しました</div>
                  <input v-model="resumeSettlementConfirmation" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
                </div>
              </div>
              <div>
                <button v-on:click="resumeSettlement" v-bind:disabled="!resumeSettlementConfirmation" class="mt-4 ml-2 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">決済を再開する</button>
              </div>
              <div v-if="resumeSettlementErrMessage" class="mt-4">
                <AlertMessage v-bind:message="resumeSettlementErrMessage"/>
              </div>
            </div>
          </div>
          <div v-else class="m-4 text-2xl">
            停止中の決済情報は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="stoppedSettlementErrMessage"/>
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
import { Settlement } from '@/util/personalized/consultation/settlement/Settlement'
import { useSettlementByConsultationId } from '@/util/personalized/consultation/settlement/useGetSettlementByConsultationId'
import { GetSettlementByConsultationIdResp } from '@/util/personalized/consultation/settlement/GetSettlementByConsultationIdResp'
import { useStoppedSettlementByConsultationId } from '@/util/personalized/consultation/stopped_settlement/useGetStoppedSettlementByConsultationId'
import { GetStoppedSettlementByConsultationIdResp } from '@/util/personalized/consultation/stopped_settlement/GetStoppeSettlementByConsultationIdResp'
import { StoppedSettlement } from '@/util/personalized/consultation/stopped_settlement/StoppedSettlement'
import { Receipt } from '@/util/personalized/consultation/receipt/Receipt'
import { useReceiptByConsultationId } from '@/util/personalized/consultation/receipt/useGetReceiptByConsultationId'
import { GetReceiptByConsultationIdResp } from '@/util/personalized/consultation/receipt/GetReceiptByConsultationIdResp'
import { Refund } from '@/util/personalized/consultation/refund/Refund'
import { useGetRefundByConsultationId } from '@/util/personalized/consultation/refund/useGetRefundByConsultationId'
import { GetRefundByConsultationIdResp } from '@/util/personalized/consultation/refund/GetRefundByConsultationIdResp'
import { usePostStopSettlementReq } from '@/util/personalized/consultation/stop_settlement_req/usePostStopSettlementReq'
import { PostStopSettlementReqResp } from '@/util/personalized/consultation/stop_settlement_req/PostStopSettlementReqResp'
import { usePostResumeSettlementReq } from '@/util/personalized/consultation/resume_settlement_req/usePostResumeSettlementReq'
import { PostResumeSettlementReqResp } from '@/util/personalized/consultation/resume_settlement_req/PostResumeSettlementReqResp'
import { usePostMakePaymentReq } from '@/util/personalized/consultation/make_payment_req/usePostMakePaymentReq'
import { PostMakePaymentReqResp } from '@/util/personalized/consultation/make_payment_req/PostMakePaymentReqResp'
import { usePostRefundReq } from '@/util/personalized/consultation/refund_req/usePostRefundReq'
import { PostRefundReqResp } from '@/util/personalized/consultation/refund_req/PostRefundReqResp'

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

    const settlement = ref(null as Settlement | null)
    const settlementErrMessage = ref(null as string | null)

    const {
      getSettlementByConsultationIdDone,
      getSettlementByConsultationIdFunc
    } = useSettlementByConsultationId()

    const findSettlement = async () => {
      try {
        const response = await getSettlementByConsultationIdFunc(consultationId)
        if (!(response instanceof GetSettlementByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          settlementErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getSettlementResult()
        settlement.value = result.settlement
      } catch (e) {
        settlementErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const stoppedSettlement = ref(null as StoppedSettlement | null)
    const stoppedSettlementErrMessage = ref(null as string | null)

    const {
      getStoppedSettlementByConsultationIdDone,
      getStoppedSettlementByConsultationIdFunc
    } = useStoppedSettlementByConsultationId()

    const findStoppedSettlement = async () => {
      try {
        const response = await getStoppedSettlementByConsultationIdFunc(consultationId)
        if (!(response instanceof GetStoppedSettlementByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          stoppedSettlementErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getStoppedSettlementResult()
        stoppedSettlement.value = result.stopped_settlement
      } catch (e) {
        stoppedSettlementErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
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
      postStopSettlementReqDone,
      postStopSettlementReqFunc
    } = usePostStopSettlementReq()

    const stopSettlementConfirmation = ref(false)
    const stopSettlementErrMessage = ref(null as string | null)

    const stopSettlement = async () => {
      if (!settlement.value) {
        stopSettlementErrMessage.value = `${Message.UNEXPECTED_ERR}: settlement.value is null`
        return
      }
      const settlementId = settlement.value.settlement_id
      try {
        const response = await postStopSettlementReqFunc(settlementId)
        if (!(response instanceof PostStopSettlementReqResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          stopSettlementErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        stopSettlementErrMessage.value = null
      } catch (e) {
        stopSettlementErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      } finally {
        stopSettlementConfirmation.value = false
      }
      await findSettlement()
      await findStoppedSettlement()
    }

    const {
      postResumeSettlementReqDone,
      postResumeSettlementReqFunc
    } = usePostResumeSettlementReq()

    const resumeSettlementConfirmation = ref(false)
    const resumeSettlementErrMessage = ref(null as string | null)

    const resumeSettlement = async () => {
      if (!stoppedSettlement.value) {
        resumeSettlementErrMessage.value = `${Message.UNEXPECTED_ERR}: stoppedSettlement.value is null`
        return
      }
      const stoppedSettlementId = stoppedSettlement.value.stopped_settlement_id
      try {
        const response = await postResumeSettlementReqFunc(stoppedSettlementId)
        if (!(response instanceof PostResumeSettlementReqResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          resumeSettlementErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        resumeSettlementErrMessage.value = null
      } catch (e) {
        resumeSettlementErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      } finally {
        resumeSettlementConfirmation.value = false
      }
      await findSettlement()
      await findStoppedSettlement()
    }

    const {
      postMakePaymentReqDone,
      postMakePaymentReqFunc
    } = usePostMakePaymentReq()

    const makePaymentConfirmation = ref(false)
    const makePaymentErrMessage = ref(null as string | null)

    const makePayment = async () => {
      if (!settlement.value) {
        makePaymentErrMessage.value = `${Message.UNEXPECTED_ERR}: settlement.value is null`
        return
      }
      const settlementId = settlement.value.settlement_id
      try {
        const response = await postMakePaymentReqFunc(settlementId)
        if (!(response instanceof PostMakePaymentReqResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          makePaymentErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        makePaymentErrMessage.value = null
      } catch (e) {
        makePaymentErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      } finally {
        makePaymentConfirmation.value = false
      }
      await findSettlement()
      await findReceipt()
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
      await findSettlement()
      await findStoppedSettlement()
      await findReceipt()
      await findRefund()
    })

    const requestsDone = computed(() => {
      return getConsultationByConsultationIdDone.value &&
              getUserRatingByConsultationIdDone.value &&
              getConsultantRatingByConsultationIdDone.value &&
              getSettlementByConsultationIdDone.value &&
              getStoppedSettlementByConsultationIdDone.value &&
              getReceiptByConsultationIdDone.value &&
              getRefundByConsultationIdDone.value &&
              postStopSettlementReqDone.value &&
              postResumeSettlementReqDone.value &&
              postMakePaymentReqDone.value &&
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
      settlement,
      settlementErrMessage,
      stoppedSettlement,
      stoppedSettlementErrMessage,
      receipt,
      receiptErrMessage,
      refund,
      refundErrMessage,
      stopSettlementConfirmation,
      stopSettlement,
      stopSettlementErrMessage,
      resumeSettlementConfirmation,
      resumeSettlement,
      resumeSettlementErrMessage,
      makePaymentConfirmation,
      makePayment,
      makePaymentErrMessage,
      refundReqConfirmation,
      refundReq,
      refundReqErrMessage
    }
  }
})
</script>
