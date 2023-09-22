<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getRewardsDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="errorExists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="errorMessage"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-xl lg:text-2xl">報酬の入金口座</h3>
          <p class="mt-2 text-base lg:text-xl">受け取った報酬を入金するための口座で、相談受け付けを行うために必要となる情報です。他のユーザーに公開されることはありません。ユーザー情報で本人確認が完了した姓名と異なる名義の口座は設定できません。</p>
          <div v-if="bankAccount !== null" data-test="bank-account-set" class="m-4 text-xl lg:text-2xl grid grid-cols-2">
            <div class="justify-self-start col-span-1">銀行コード</div><div class="justify-self-start col-span-1">{{ bankAccount.bank_code }}</div>
            <div class="justify-self-start col-span-1">支店コード</div><div class="justify-self-start col-span-1">{{ bankAccount.branch_code }}</div>
            <div class="justify-self-start col-span-1">預金種別</div><div class="justify-self-start col-span-1">{{ bankAccount.account_type }}</div>
            <div class="justify-self-start col-span-1">口座番号</div><div class="justify-self-start col-span-1">{{ bankAccount.account_number }}</div>
            <div class="justify-self-start col-span-1">口座名義</div><div class="justify-self-start col-span-1">{{ bankAccount.account_holder_name }}</div>
          </div>
          <p v-else data-test="no-bank-account-set" class="m-4 text-base lg:text-xl">報酬の入金口座が設定されていません。</p>
          <button v-on:click="moveToBankAccountPage" data-test="move-to-bank-account-page-button" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">報酬の入金口座を編集する</button>
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
import { Message } from '@/util/Message'
import { Code, createErrorMessage } from '@/util/Error'
import { GetRewardsResp } from '@/util/personalized/reward/GetRewardsResp'
import { useStore } from 'vuex'
import { SET_BANK_ACCOUNT } from '@/store/mutationTypes'

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
      errorExists,
      errorMessage,
      moveToBankAccountPage
    }
  }
})
</script>
