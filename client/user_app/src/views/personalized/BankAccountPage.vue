<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="postBankAccountDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">報酬の入金口座</h3>
        <p class="mt-2 text-lg">受け取った報酬を入金するための口座で、相談受け付けを行うために必要となる情報です。他のユーザーに公開されることはありません。ユーザー情報で本人確認が完了した姓名と異なる名義の口座は設定できません。</p>
        <form @submit.prevent="submitBankAccount">
          <!-- <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              氏名
            </div>
            <div data-test="last-name-div" class="mt-2 justify-self-start col-span-3 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">姓</label>
              <input v-bind:value="form.lastName" v-on:input="setLastName" type="text" required minlength="1" maxlength="64" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="first-name-div" class="mt-2 ml-1 justify-self-start col-span-3 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">名</label>
              <input v-bind:value="form.firstName" v-on:input="setFirstName" type="text" required minlength="1" maxlength="64" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              フリガナ
            </div>
            <div data-test="last-name-furigana-div" class="mt-2 justify-self-start col-span-3 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">セイ</label>
              <input v-bind:value="form.lastNameFurigana" v-on:input="setLastNameFurigana" type="text" required minlength="1" maxlength="64" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="first-name-furigana-div" class="mt-2 ml-1 justify-self-start col-span-3 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">メイ</label>
              <input v-bind:value="form.firstNameFurigana" v-on:input="setFirstNameFurigana" type="text" required minlength="1" maxlength="64" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="tel-div" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              電話番号（ハイフンを含めず、半角数字のみで入力）
            </div>
            <div data-test="tel-input-div" class="mt-2 w-full justify-self-start col-span-6 pt-3 pl-2 rounded bg-gray-200">
              <input v-bind:value="form.telephoneNumber" v-on:input="setTelephoneNumber" type="text" inputmode="tel" pattern="[0-9]{10,13}" title="半角数字のみで10桁以上13桁以下でご入力下さい。" required minlength="10" maxlength="13" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
          </div> -->
          <button data-test="submit-button" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">本人確認を依頼する</button>
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
    const postBankAccountDone = ref(true)
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
      console.log('bank account')
    }
    return {
      error,
      bankAccount,
      postBankAccountDone,
      submitBankAccount
    }
  }
})
</script>
