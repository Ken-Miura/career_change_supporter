<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">test</h3>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
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
    TheHeader
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
        const resp = await deleteAccountFunc()
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
      deleteAccount
    }
  }
})
</script>
