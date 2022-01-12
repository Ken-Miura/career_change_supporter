<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">ユーザー情報</h3>
        <p class="mt-2 text-lg">身分証明のために入力する情報で、相談申し込みを行うために必要となる情報です。下記の項目を入力し、本人確認を依頼すると、審査後に入力した値が反映されます。ユーザー情報が他のユーザーに公開されることはありません。</p>
        <form @submit.prevent="submitIdentity">
          <div class="m-4 text-2xl grid grid-cols-4">
            <div class="mt-2 justify-self-start col-span-2 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">名前（姓）</label>
              <input v-bind:value="form.lastName" v-on:input="setLastName" type="text" required minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 justify-self-start col-span-2 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">名前（名）</label>
              <input v-bind:value="form.firstName" v-on:input="setFirstName" type="text" required minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
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
import { checkAgreementStatus } from '@/util/agreement-status/CheckAgreementStatus'
import { CheckAgreementStatusResp } from '@/util/agreement-status/CheckAgreementStatusResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'

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
    const { form, setLastName, setFirstName } = useIdentity()
    onMounted(async () => {
      try {
        const agreementStatus = await checkAgreementStatus()
        if (agreementStatus instanceof CheckAgreementStatusResp) {
          // セッションが存在し、利用規約に同意済のため、ログイン後のページを表示可能
          // TODO: 正常系の処理
          // 表示する際の初期値として使いたいだけなので、identityはrefとして宣言しない（リアクティブとしない）
          const identity = store.state.identity
          if (identity !== null) {
            /* eslint-disable camelcase */
            form.lastName = identity.last_name
            form.firstName = identity.first_name
            /* eslint-enable camelcase */
          }
        } else if (agreementStatus instanceof ApiErrorResp) {
          const code = agreementStatus.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('terms-of-use')
            return
          }
          // TODO: エラー処理
        }
      } catch (e) {
        // TODO: エラー処理
      }
      console.log('TODO: 実装後削除')
    })
    const submitIdentity = async () => {
      console.log(form.lastName)
      console.log(form.firstName)
    }
    return { isHidden, errorMessage, form, setLastName, setFirstName, submitIdentity }
  }
})
</script>
