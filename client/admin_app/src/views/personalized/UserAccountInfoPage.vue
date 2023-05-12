<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="false" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="outerErrorMessage" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <AlertMessage v-bind:message="outerErrorMessage"/>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">検索条件</h3>
          <p v-if="accountId" class="mt-4 ml-4 text-xl">アカウントID: {{ accountId }}</p>
          <p v-else-if="emailAddress" class="mt-4 ml-4 text-xl">メールアドレス: {{ emailAddress }}</p>
          <p v-else class="mt-4 ml-4 text-xl">意図しない動作です。管理者に連絡して下さい</p>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          {{ accountId }}, {{ emailAddress }}
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex flex-col text-white">
      <router-link to="/admin-menu" class="hover:underline text-center">管理メニューへ</router-link>
      <router-link to="/" class="mt-6 hover:underline text-center">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useStore } from 'vuex'
import { UserAccountSearchParam } from '@/util/personalized/user-account-search/UserAccountSearchParam'
import { Message } from '@/util/Message'

export default defineComponent({
  name: 'UserAccountInfoPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const store = useStore()

    const accountId = ref(null as number | null)
    const emailAddress = ref(null as string | null)

    const outerErrorMessage = ref(null as string | null)

    onMounted(async () => {
      const param = store.state.userAccountSearchParam as UserAccountSearchParam
      if (!param) {
        outerErrorMessage.value = Message.USER_ACCOUNT_SEARCH_PARAM_IS_NULL
        return
      }
      if (param.accountId === null && param.emailAddress === null) {
        outerErrorMessage.value = Message.BOTH_ACCOUNT_ID_AND_EMAIL_ADDRESS_ARE_EMPTY_MESSAGE
        return
      }
      if (param.accountId !== null && param.emailAddress !== null) {
        outerErrorMessage.value = Message.BOTH_ACCOUNT_ID_AND_EMAIL_ADDRESS_ARE_FILLED_MESSAGE
        return
      }
      accountId.value = param.accountId
      emailAddress.value = param.emailAddress
    })

    return {
      accountId,
      emailAddress,
      outerErrorMessage
    }
  }
})
</script>
