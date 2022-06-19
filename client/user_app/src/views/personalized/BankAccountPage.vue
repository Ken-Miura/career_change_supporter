<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="postBankAccountDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">報酬の入金口座</h3>
        <form @submit.prevent="submitBankAccount">
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              銀行コード
            </div>
            <div data-test="bank-code-div" class="mt-2 justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="bankAccount.bank_code" v-on:input="setBankCode" type="text" required minlength="4" maxlength="4" pattern="\d*" title="半角数字4桁でご入力下さい。" class="text-right bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              支店コード
            </div>
            <div data-test="branch-code-div" class="mt-2 justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="bankAccount.branch_code" v-on:input="setBranchCode" type="text" required minlength="3" maxlength="3" pattern="\d*" title="半角数字3桁でご入力下さい。" class="text-right bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              預金種別（預金種別は普通のみサポートしております）
            </div>
            <div data-test="account-type-div" class="mt-2 justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <label class="text-right bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">{{ bankAccount.account_type }}</label>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              口座番号
            </div>
            <div data-test="account-number-div" class="mt-2 justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="bankAccount.account_number" v-on:input="setAccountNumber" type="text" required minlength="7" maxlength="8" pattern="\d*" title="半角数字7桁または8桁でご入力下さい。" class="text-right bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              口座名義（全角カタカナと全角空白のみを使い、セイとメイの間に全角空白を入れて下さい）
            </div>
            <div data-test="account-holder-name-div" class="mt-2 justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="bankAccount.account_holder_name" v-on:input="setAccountHolderName" type="text" required minlength="3" maxlength="129" pattern="^[ァ-ヴー　]+$" title="全角カタカナと全角空白のみで、3文字以上129文字以内でご入力下さい。" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
          </div>
          <button data-test="submit-button" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">報酬の入金口座を設定する</button>
          <AlertMessage v-bind:class="['mt-6', { 'hidden': !error.exists }]" v-bind:message="error.message"/>
        </form>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { Message } from '@/util/Message'
import { useStore } from 'vuex'
import { BankAccount } from '@/util/personalized/BankAccount'
import { usePostBankAccount } from '@/util/personalized/bank-account/usePostBankAccount'
import { PostBankAccountResp } from '@/util/personalized/bank-account/PostBankAccountResp'

export default defineComponent({
  name: 'BankAccountPage',
  components: {
    TheHeader,
    WaitingCircle,
    AlertMessage
  },
  setup () {
    const error = reactive({
      exists: false,
      message: ''
    })
    const bankAccount = ref({
      bank_code: '',
      branch_code: '',
      account_type: '普通',
      account_number: '',
      account_holder_name: ''
    } as BankAccount)
    const {
      postBankAccountDone,
      postBankAccountFunc
    } = usePostBankAccount()
    const router = useRouter()
    const store = useStore()
    onMounted(async () => {
      try {
        const resp = await refresh()
        if (!(resp instanceof RefreshResp)) {
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
        const value = store.state.bankAccount
        if (!value) {
          return
        }
        bankAccount.value = value
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })
    const submitBankAccount = async () => {
      try {
        const response = await postBankAccountFunc(bankAccount.value)
        if (!(response instanceof PostBankAccountResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          }
          error.exists = true
          error.message = createErrorMessage(response.getApiError().getCode())
          return
        }
        await router.push('/submit-bank-account-success')
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const setBankCode = (e: Event) => {
      const target = (e && e.target)
      if (!(target instanceof HTMLInputElement)) {
        throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
      }
      bankAccount.value.bank_code = target.value
    }

    const setBranchCode = (e: Event) => {
      const target = (e && e.target)
      if (!(target instanceof HTMLInputElement)) {
        throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
      }
      bankAccount.value.branch_code = target.value
    }

    const setAccountNumber = (e: Event) => {
      const target = (e && e.target)
      if (!(target instanceof HTMLInputElement)) {
        throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
      }
      bankAccount.value.account_number = target.value
    }

    const setAccountHolderName = (e: Event) => {
      const target = (e && e.target)
      if (!(target instanceof HTMLInputElement)) {
        throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
      }
      bankAccount.value.account_holder_name = target.value
    }

    return {
      error,
      bankAccount,
      postBankAccountDone,
      submitBankAccount,
      setBankCode,
      setBranchCode,
      setAccountNumber,
      setAccountHolderName
    }
  }
})
</script>
