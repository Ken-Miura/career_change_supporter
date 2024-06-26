<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
      <h3 data-test="label" class="font-bold text-2xl">アカウント検索</h3>
      <p data-test="description" class="ml-2 mt-4 mb-6 text-lg">アカウントID、またはメールアドレスを入力して検索を押して下さい。既に削除されたアカウントは、メールアドレスではなくアカウントIDで検索して下さい。削除されたアカウントは、そうでないアカウントと比較し、確認できる情報が限定されています。</p>
      <form class="flex flex-col" @submit.prevent="searchUserAccountHandler">
        <div class="pt-3 rounded bg-gray-200">
          <label data-test="account-id-label" class="block text-gray-700 text-sm font-bold mb-2 ml-3">アカウントID</label>
          <input data-test="account-id-value" v-model="accountId" minlength="1" pattern="\d+" title="数値" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
        </div>
        <div class="ml-2 p-2 text-xl">
          or
        </div>
        <div class="mb-6 pt-3 rounded bg-gray-200">
          <label data-test="email-address-label" class="block text-gray-700 text-sm font-bold mb-2 ml-3">メールアドレス</label>
          <input data-test="email-address-value" v-model="emailAddress" type="email" minlength="1" maxlength="254" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
        </div>
        <button data-test="button" v-bind:disabled="disabled" class="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none" type="submit">検索</button>
        <div v-if="errorMessage">
          <AlertMessage class="mt-6" v-bind:message="errorMessage"/>
        </div>
      </form>
    </main>
    <footer class="max-w-lg mx-auto flex flex-col text-white">
      <router-link to="/admin-menu" class="hover:underline text-center">管理メニューへ</router-link>
      <router-link to="/" class="mt-6 hover:underline text-center">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { useRouter } from 'vue-router'
import { useStore } from 'vuex'
import { UserAccountSearchParam } from '@/util/personalized/user-account-search/UserAccountSearchParam'
import { SET_USER_ACCOUNT_SEARCH_PARAM } from '@/store/mutationTypes'
import { Message } from '@/util/Message'

export default defineComponent({
  name: 'UserAccountSearchPage',
  components: {
    TheHeader,
    AlertMessage
  },
  setup () {
    const store = useStore()
    const router = useRouter()

    const errorMessage = ref(null as string | null)
    const accountId = ref('')
    const emailAddress = ref('')

    const bothEmpty = () => {
      return accountId.value === '' && emailAddress.value === ''
    }

    const bothFilled = () => {
      return accountId.value !== '' && emailAddress.value !== ''
    }

    const disabled = computed(() => {
      return bothEmpty() || bothFilled()
    })

    const createUserAccountSearchParam = (accountId: string, emailAddress: string) => {
      let acccountIdNumber = null
      if (accountId !== '') {
        acccountIdNumber = parseInt(accountId)
      }
      let emailAddr = null
      if (emailAddress !== '') {
        emailAddr = emailAddress
      }
      return {
        accountId: acccountIdNumber,
        emailAddress: emailAddr
      } as UserAccountSearchParam
    }

    const searchUserAccountHandler = async () => {
      if (bothEmpty()) {
        errorMessage.value = Message.BOTH_ACCOUNT_ID_AND_EMAIL_ADDRESS_ARE_EMPTY_MESSAGE
        return
      }
      if (bothFilled()) {
        errorMessage.value = Message.BOTH_ACCOUNT_ID_AND_EMAIL_ADDRESS_ARE_FILLED_MESSAGE
        return
      }

      const param = createUserAccountSearchParam(accountId.value, emailAddress.value)
      store.commit(SET_USER_ACCOUNT_SEARCH_PARAM, param)
      await router.push('/user-account-info')
    }

    return {
      accountId,
      emailAddress,
      disabled,
      errorMessage,
      searchUserAccountHandler
    }
  }
})
</script>
