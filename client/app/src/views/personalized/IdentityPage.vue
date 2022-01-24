<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">ユーザー情報</h3>
        <p class="mt-2 text-lg">身分証明のために入力する情報で、相談申し込みを行うために必要となる情報です。下記の項目を入力し、本人確認を依頼すると、審査後に入力した値が反映されます。ユーザー情報が他のユーザーに公開されることはありません。</p>
        <form @submit.prevent="submitIdentity">
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              氏名
            </div>
            <div class="mt-2 justify-self-start col-span-3 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">姓</label>
              <input v-bind:value="form.lastName" v-on:input="setLastName" type="text" required minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 ml-1 justify-self-start col-span-3 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">名</label>
              <input v-bind:value="form.firstName" v-on:input="setFirstName" type="text" required minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              フリガナ
            </div>
            <div class="mt-2 justify-self-start col-span-3 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">セイ</label>
              <input v-bind:value="form.lastNameFurigana" v-on:input="setLastNameFurigana" type="text" required minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 ml-1 justify-self-start col-span-3 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">メイ</label>
              <input v-bind:value="form.firstNameFurigana" v-on:input="setFirstNameFurigana" type="text" required minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              性別
            </div>
            <div class="mt-2 w-5/6 text-2xl justify-self-start col-span-6">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option>男性</option>
                <option>女性</option>
              </select>
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              生年月日
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option>1990</option>
                <option>1989</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              年
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option>11</option>
                <option>12</option>
              </select>
            </div>
            <div class="mt-7 text-2xl justify-self-start col-span-1 pt-3">
              月
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option>30</option>
                <option>31</option>
              </select>
            </div>
            <div class="mt-7 text-2xl justify-self-start col-span-1 pt-3">
              日
            </div>
          </div>
          <button class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">本人確認を依頼する</button>
          <AlertMessage v-bind:class="['mt-6', { 'hidden': isHidden }]" v-bind:message="errorMessage"/>
        </form>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import { useStore } from 'vuex'
import { useIdentity } from '@/views/personalized/useIdentity'
import TheHeader from '@/components/TheHeader.vue'
import { useRouter } from 'vue-router'
import AlertMessage from '@/components/AlertMessage.vue'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { Message } from '@/util/Message'

export default defineComponent({
  name: 'IdentityPage',
  components: {
    TheHeader,
    AlertMessage
  },
  setup () {
    const router = useRouter()
    const store = useStore()
    const isHidden = ref(true)
    const errorMessage = ref('')
    const {
      form,
      setLastName,
      setFirstName,
      setLastNameFurigana,
      setFirstNameFurigana
    } = useIdentity()
    onMounted(async () => {
      try {
        const resp = await refresh()
        if (resp instanceof RefreshResp) {
          // セッションが存在し、利用規約に同意済のため、ログイン後のページを表示可能
          // TODO: 正常系の処理
          // 表示する際の初期値として使いたいだけなので、identityはrefとして宣言しない（リアクティブとしない）
          const identity = store.state.identity
          if (identity !== null) {
            /* eslint-disable camelcase */
            form.lastName = identity.last_name
            form.firstName = identity.first_name
            form.lastNameFurigana = identity.last_name_furigana
            form.firstNameFurigana = identity.first_name_furigana
            /* eslint-enable camelcase */
          }
        } else if (resp instanceof ApiErrorResp) {
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('terms-of-use')
            return
          }
          throw new Error(`unexpected result: ${resp}`)
        } else {
          throw new Error(`unexpected result: ${resp}`)
        }
      } catch (e) {
        isHidden.value = false
        errorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })
    const submitIdentity = async () => {
      console.log(form.lastName)
      console.log(form.firstName)
      console.log(form.lastNameFurigana)
      console.log(form.firstNameFurigana)
    }
    return {
      isHidden,
      errorMessage,
      form,
      setLastName,
      setFirstName,
      setLastNameFurigana,
      setFirstNameFurigana,
      submitIdentity
    }
  }
})
</script>
