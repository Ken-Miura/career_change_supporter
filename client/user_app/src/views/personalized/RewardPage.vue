<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getRewardsDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="errorExists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="errorMessage"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">報酬の入金口座</h3>
          <p class="mt-2 text-lg">受け取った報酬を入金するための口座で、相談受け付けを行うために必要となる情報です。他のユーザーに公開されることはありません。ユーザー情報で本人確認が完了した姓名と異なる名義の口座は設定できません。</p>
          <div v-if="bankAccount !== null" data-test="bank-account-set" class="m-4 text-2xl grid grid-cols-3">
            <div class="mt-2 justify-self-start col-span-1">銀行コード</div><div class="justify-self-start col-span-2">{{ bankAccount.bank_code }}</div>
            <div class="mt-2 justify-self-start col-span-1">支店コード</div><div class="justify-self-start col-span-2">{{ bankAccount.branch_code }}</div>
            <div class="mt-2 justify-self-start col-span-1">預金種別</div><div class="justify-self-start col-span-2">{{ bankAccount.account_type }}</div>
            <div class="mt-2 justify-self-start col-span-1">口座番号</div><div class="justify-self-start col-span-2">{{ bankAccount.account_number }}</div>
            <div class="mt-2 justify-self-start col-span-1">口座名義</div><div class="justify-self-start col-span-2">{{ bankAccount.account_holder_name }}</div>
          </div>
          <p v-else data-test="no-bank-account-set" class="m-4 text-xl">報酬の入金口座が設定されていません。</p>
          <button v-on:click="moveToBankAccountPage" data-test="move-to-bank-account-page-button" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">報酬の入金口座を編集する</button>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">今月の報酬の合計</h3>
          <p class="mt-2 text-lg">今月受け付けし、承諾した相談の報酬の合計です。他のユーザーに公開されることはありません。</p>
          <div v-if="rewardsOfTheMonth !== null" data-test="rewards-of-the-month-set" class="flex justify-end">
            <p class="m-4 text-2xl">{{ rewardsOfTheMonth }}円</p>
          </div>
          <p v-else data-test="no-rewards-of-the-month-set" class="m-4 text-xl">まだ相談を受け付けていません。</p>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">入金情報</h3>
          <p data-test="latest-two-transfers-set-description" class="mt-2 text-lg">報酬に関する直近二回分の入金情報です。毎月月末に、前月の報酬の合計から振込手数料（{{ TRANSFER_FEE_IN_YEN }}円）が差し引かれた金額が入金されます。他のユーザーに公開されることはありません。</p>
          <div v-if="latestTwoTransfers.length === 0" data-test="no-latest-two-transfers-set" class="mt-4 ml-4 text-xl">入金情報はありません。</div>
          <div v-else data-test="latest-two-transfers-set">
            <ul>
              <li v-for="(transfer, index) in latestTwoTransfers" v-bind:key="transfer.transfer_id">
                <div class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">入金情報{{ index + 1 }}</div>
                  <div v-if="transfer.status === 'pending'">
                    <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                      <div class="mt-2 justify-self-start col-span-1">処理状態</div><div class="justify-self-start col-span-2">入金前</div>
                      <div class="mt-2 justify-self-start col-span-1">入金予定額</div><div v-if="transfer.carried_balance !== null" class="justify-self-start col-span-2">{{ transfer.amount + transfer.carried_balance - TRANSFER_FEE_IN_YEN }}円</div><div v-else class="justify-self-start col-span-2">{{ transfer.amount - TRANSFER_FEE_IN_YEN }}円</div>
                      <div class="mt-2 justify-self-start col-span-1">入金予定日</div><div class="justify-self-start col-span-2">{{ transfer.scheduled_date_in_jst.year }}年{{ transfer.scheduled_date_in_jst.month }}月{{ transfer.scheduled_date_in_jst.day }}日</div>
                    </div>
                  </div>
                  <div v-else-if="transfer.status === 'paid'">
                    <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                      <div class="mt-2 justify-self-start col-span-1">処理状態</div><div class="justify-self-start col-span-2">入金完了</div>
                      <div class="mt-2 justify-self-start col-span-1">入金予定額</div><div v-if="transfer.carried_balance !== null" class="justify-self-start col-span-2">{{ transfer.amount + transfer.carried_balance - TRANSFER_FEE_IN_YEN }}円</div><div v-else class="justify-self-start col-span-2">{{ transfer.amount - TRANSFER_FEE_IN_YEN }}円</div>
                      <div class="mt-2 justify-self-start col-span-1">入金予定日</div><div class="justify-self-start col-span-2">{{ transfer.scheduled_date_in_jst.year }}年{{ transfer.scheduled_date_in_jst.month }}月{{ transfer.scheduled_date_in_jst.day }}日</div>
                      <div v-if="transfer.transfer_amount !== null" class="mt-2 justify-self-start col-span-1">入金額</div><div v-if="transfer.transfer_amount !== null" class="justify-self-start col-span-2">{{ transfer.transfer_amount }}円</div>
                      <div v-if="transfer.transfer_amount === null" class="mt-2 justify-self-start col-span-1">入金額</div><div v-if="transfer.transfer_amount === null" class="mt-2 justify-self-start col-span-2">入金額が正しく表示出来ませんでした。お手数ですが、<router-link class="font-bold" to="/transaction-law">お問い合わせ先</router-link>より問題のご報告をお願いいたします。</div>
                      <div v-if="transfer.transfer_date_in_jst !== null" class="mt-2 justify-self-start col-span-1">入金日</div><div v-if="transfer.transfer_date_in_jst !== null" class="justify-self-start col-span-2">{{ transfer.transfer_date_in_jst.year }}年{{ transfer.transfer_date_in_jst.month }}月{{ transfer.transfer_date_in_jst.day }}日</div>
                      <div v-if="transfer.transfer_date_in_jst === null" class="mt-2 justify-self-start col-span-1">入金日</div><div v-if="transfer.transfer_date_in_jst === null" class="mt-2 justify-self-start col-span-2">入金日が正しく表示出来ませんでした。お手数ですが、<router-link class="font-bold" to="/transaction-law">お問い合わせ先</router-link>より問題のご報告をお願いいたします。</div>
                    </div>
                  </div>
                  <div v-else-if="transfer.status === 'recombination' || transfer.status === 'failed'">
                    <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                      <div class="mt-2 justify-self-start col-span-1">処理状態</div><div class="justify-self-start col-span-2">入金失敗（次回の入金時までに報酬の入金口座を正しい情報で登録し直してください）</div>
                      <div class="mt-2 justify-self-start col-span-1">入金予定額</div><div v-if="transfer.carried_balance !== null" class="justify-self-start col-span-2">{{ transfer.amount + transfer.carried_balance - TRANSFER_FEE_IN_YEN }}円</div><div v-else class="justify-self-start col-span-2">{{ transfer.amount - TRANSFER_FEE_IN_YEN }}円</div>
                      <div class="mt-2 justify-self-start col-span-1">入金予定日</div><div class="justify-self-start col-span-2">{{ transfer.scheduled_date_in_jst.year }}年{{ transfer.scheduled_date_in_jst.month }}月{{ transfer.scheduled_date_in_jst.day }}日</div>
                    </div>
                  </div>
                  <div v-else-if="transfer.status === 'stop'">
                    <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                      <div class="mt-2 justify-self-start col-span-1">処理状態</div><div class="justify-self-start col-span-2">入金差し止め（詳細な情報は、<router-link class="font-bold" to="/transaction-law">お問い合わせ先</router-link>よりお問い合わせ下さい）</div>
                      <div class="mt-2 justify-self-start col-span-1">入金予定額</div><div v-if="transfer.carried_balance !== null" class="justify-self-start col-span-2">{{ transfer.amount + transfer.carried_balance - TRANSFER_FEE_IN_YEN }}円</div><div v-else class="justify-self-start col-span-2">{{ transfer.amount - TRANSFER_FEE_IN_YEN }}円</div>
                      <div class="mt-2 justify-self-start col-span-1">入金予定日</div><div class="justify-self-start col-span-2">{{ transfer.scheduled_date_in_jst.year }}年{{ transfer.scheduled_date_in_jst.month }}月{{ transfer.scheduled_date_in_jst.day }}日</div>
                    </div>
                  </div>
                  <div v-else-if="transfer.status === 'carried_over'">
                    <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                      <div class="mt-2 justify-self-start col-span-1">処理状態</div><div class="justify-self-start col-span-2">入金繰り越し（入金額が少額のため、次回の入金に繰り越されます）</div>
                      <div class="mt-2 justify-self-start col-span-1">繰り越し予定額</div><div class="justify-self-start col-span-2">{{ transfer.amount }}円</div>
                    </div>
                  </div>
                  <div v-else>
                    <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                      <div class="mt-2 justify-self-start col-span-1">処理状態</div><div class="justify-self-start col-span-2">想定されない処理状態（こちらの状態が表示された場合、お手数ですが<router-link class="font-bold" to="/transaction-law">お問い合わせ先</router-link>より、その旨ご連絡下さい）</div>
                    </div>
                  </div>
                </div>
              </li>
            </ul>
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
import { defineComponent, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { ApiErrorResp } from '@/util/ApiError'
import { useGetRewards } from '@/util/personalized/reward/useGetRewards'
import { BankAccount } from '@/util/personalized/BankAccount'
import { Transfer } from '@/util/personalized/reward/Transfer'
import { Message } from '@/util/Message'
import { Code, createErrorMessage } from '@/util/Error'
import { GetRewardsResp } from '@/util/personalized/reward/GetRewardsResp'
import { useStore } from 'vuex'
import { SET_BANK_ACCOUNT } from '@/store/mutationTypes'
import { TRANSFER_FEE_IN_YEN } from '@/util/personalized/reward/TransferFee'

export default defineComponent({
  name: 'RewardPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const { getRewardsDone, getRewardsFunc } = useGetRewards()
    const bankAccount = ref(null as BankAccount | null)
    const rewardsOfTheMonth = ref(null as number | null)
    const latestTwoTransfers = ref([] as Transfer[])
    const router = useRouter()
    const store = useStore()
    const errorExists = ref(false)
    const errorMessage = ref('')
    onMounted(async () => {
      try {
        const response = await getRewardsFunc()
        if (response instanceof GetRewardsResp) {
          const rewards = response.getRewards()
          /* eslint-disable camelcase */
          bankAccount.value = rewards.bank_account
          rewardsOfTheMonth.value = rewards.rewards_of_the_month
          latestTwoTransfers.value = rewards.latest_two_transfers
          /* eslint-enable camelcase */
          store.commit(SET_BANK_ACCOUNT, rewards.bank_account)
        } else if (response instanceof ApiErrorResp) {
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          }
          errorExists.value = true
          errorMessage.value = createErrorMessage(response.getApiError().getCode())
        } else {
          throw new Error(`unexpected result: ${response}`)
        }
      } catch (e) {
        errorExists.value = true
        errorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })
    const moveToBankAccountPage = async () => {
      // F5更新で最初にRewardPageに来た場合、ユーザー情報が設定されていてもidentityがnullとなるので
      // vuexにidentityがあるかどうかはチェックしない
      await router.push('/bank-account')
    }
    return {
      getRewardsDone,
      bankAccount,
      rewardsOfTheMonth,
      latestTwoTransfers,
      TRANSFER_FEE_IN_YEN,
      errorExists,
      errorMessage,
      moveToBankAccountPage
    }
  }
})
</script>
