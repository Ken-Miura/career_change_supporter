<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="false" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-2xl">アカウント検索</h3>
      <p class="ml-2 mt-4 mb-6 text-lg">アカウントID、またはメールアドレスを入力して検索を押して下さい。既に削除されたアカウントは、メールアドレスではなくアカウントIDで検索して下さい。削除されたアカウントは、そうでないアカウントと比較し、確認できる情報が限定されています。</p>
      <form class="flex flex-col" @submit.prevent="searchUserAccountHandler">
        <div class="pt-3 rounded bg-gray-200">
          <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">アカウントID</label>
          <input v-model="accountId" minlength="1" pattern="\d+" title="数値" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
        </div>
        <div class="ml-2 p-2 text-xl">
          or
        </div>
        <div class="mb-6 pt-3 rounded bg-gray-200">
          <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">メールアドレス</label>
          <input v-model="emailAddress" type="email" minlength="1" maxlength="254" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
        </div>
        <button v-bind:disabled="disabled" class="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none" type="submit">検索</button>
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
import WaitingCircle from '@/components/WaitingCircle.vue'

export default defineComponent({
  name: 'UserAccountSearchPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const accountId = ref('')
    const emailAddress = ref('')
    const disabled = computed(() => {
      return accountId.value === '' && emailAddress.value === ''
    })
    const errorMessage = ref(null as string | null)

    const searchUserAccountHandler = async () => {
      console.log(`${accountId.value}, ${emailAddress.value}`)
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
