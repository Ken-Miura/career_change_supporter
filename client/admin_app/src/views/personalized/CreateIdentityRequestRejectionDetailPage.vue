<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="waitingRequestDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">本人確認依頼（新規）拒否理由</h3>
        <p class="mt-2 text-lg">拒否理由を選択して依頼を拒否して下さい。適切な拒否理由がない場合、管理者にご連絡下さい。</p>
        <form @submit.prevent="submitRejectionReason">
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              拒否理由
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-6">
              <select v-model="rejectionReason" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="reason in reasonList" v-bind:key="reason" v-bind:value="reason">{{ reason }}</option>
              </select>
            </div>
          </div>
          <button data-test="submit-button" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">拒否する</button>
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
import { defineComponent, reactive, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import { useRoute, useRouter } from 'vue-router'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { createReasonList } from '@/util/personalized/create-identity-request-rejection-detail/ReasonList'
import { usePostCreateIdentityRequestRejection } from '@/util/personalized/create-identity-request-rejection-detail/usePostCreateIdentityRequestRejection'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { Message } from '@/util/Message'
import { PostCreateIdentityRequestRejectionResp } from '@/util/personalized/create-identity-request-rejection-detail/PostCreateIdentityRequestRejectionResp'

export default defineComponent({
  name: 'CreateIdentityRequestRejectionDetailPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const route = useRoute()
    const userAccountId = route.params.account_id as string
    const router = useRouter()
    const list = createReasonList()
    const rejectionReason = ref(list[0])
    const reasonList = ref(list)
    const error = reactive({
      exists: false,
      message: ''
    })
    const {
      waitingRequestDone,
      postCreateIdentityRequestRejectionFunc
    } = usePostCreateIdentityRequestRejection()
    const submitRejectionReason = async () => {
      try {
        const response = await postCreateIdentityRequestRejectionFunc(parseInt(userAccountId), rejectionReason.value)
        if (!(response instanceof PostCreateIdentityRequestRejectionResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          error.exists = true
          error.message = createErrorMessage(response.getApiError().getCode())
          return
        }
        await router.push('/create-identity-request-rejection')
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    return {
      error,
      rejectionReason,
      reasonList,
      waitingRequestDone,
      submitRejectionReason
    }
  }
})
</script>
