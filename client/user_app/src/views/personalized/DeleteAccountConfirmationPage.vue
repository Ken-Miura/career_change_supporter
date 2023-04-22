<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!(refreshDone && deleteAccountDone)" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else class="flex flex-col justify-center bg-white max-w-xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <div v-if="refreshErrorMessage">
        <AlertMessage class="mt-2" v-bind:message="refreshErrorMessage"/>
      </div>
      <div v-else>
        <h3 class="font-bold text-2xl">アカウントの削除</h3>
        <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
          <p>確認事項</p>
          <p class="mt-2 ml-2 text-lg">私は下記に記載の内容を理解した上でアカウントの削除を行います。</p>
        </div>
        <div class="mt-2 min-w-full justify-self-start col-span-6 rounded bg-gray-200">
          <div class="p-4 text-xl grid grid-cols-6 justify-center items-center">
            <div class="col-span-5">
              <ul class="ml-4 space-y-2 list-disc">
                <li data-test="first-confirmation">未入金の報酬を受け取れなくなる可能性を受け入れます。</li>
                <li data-test="second-confirmation">受け付け済みの相談を実施しないことにより問題が発生した場合、その責任を負うことを理解しています。</li>
              </ul>
            </div>
            <input v-model="accountDeleteConfirmed" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
          </div>
        </div>
        <button v-on:click="deleteAccount" v-bind:disabled="!accountDeleteConfirmed" data-test="submit-button" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none" type="submit">アカウントを削除する</button>
        <div v-if="deleteAccountErrorMessage">
          <AlertMessage class="mt-2" v-bind:message="deleteAccountErrorMessage"/>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex flex-col text-white">
      <router-link to="/profile" class="hover:underline text-center">プロフィールへ</router-link>
      <router-link to="/" class="mt-6 hover:underline text-center">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { useRefresh } from '@/util/personalized/refresh/useRefresh'
import { ApiErrorResp } from '@/util/ApiError'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { Code, createErrorMessage } from '@/util/Error'
import { Message } from '@/util/Message'
import { useRouter } from 'vue-router'
import { useDeleteAccount } from '@/util/personalized/delete-account-confirmation/useDeleteAccount'
import { DeleteAccountResp } from '@/util/personalized/delete-account-confirmation/DeleteAccountResp'

export default defineComponent({
  name: 'DeleteAccountConfirmationPage',
  components: {
    TheHeader,
    WaitingCircle,
    AlertMessage
  },
  setup () {
    const router = useRouter()

    const {
      refreshDone,
      refreshFunc
    } = useRefresh()
    const refreshErrorMessage = ref(null as string | null)

    const {
      deleteAccountDone,
      deleteAccountFunc
    } = useDeleteAccount()
    const deleteAccountErrorMessage = ref(null as string | null)

    const accountDeleteConfirmed = ref(false)

    onMounted(async () => {
      try {
        const resp = await refreshFunc()
        if (!(resp instanceof RefreshResp)) {
          if (!(resp instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${resp}`)
          }
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            // エラーメッセージ表示にする？
            // await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            // エラーメッセージ表示にする？
            // await router.push('/terms-of-use')
            return
          }
          refreshErrorMessage.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
      } catch (e) {
        refreshErrorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const deleteAccount = async () => {
      try {
        const resp = await deleteAccountFunc(false)
        if (!(resp instanceof DeleteAccountResp)) {
          if (!(resp instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${resp}`)
          }
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            // エラーメッセージ表示にする？
            // await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            // エラーメッセージ表示にする？
            // await router.push('/terms-of-use')
            return
          }
          deleteAccountErrorMessage.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
      } catch (e) {
        deleteAccountErrorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    return {
      refreshDone,
      refreshErrorMessage,
      deleteAccountDone,
      deleteAccountErrorMessage,
      deleteAccount,
      accountDeleteConfirmed
    }
  }
})
</script>
