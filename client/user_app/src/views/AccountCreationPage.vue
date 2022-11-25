<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="flex justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <div v-if="error.exists">
        <AlertMessage class="mt-2" v-bind:message="error.message"/>
      </div>
      <div v-else>
        <h3 class="font-bold text-lg">{{ message }}</h3>
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
import { createAccount } from '@/util/account/CreateAccount'
import { CreateAccountResp } from '@/util/account/CreateAccountResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'
import { createErrorMessage } from '@/util/Error'
import AlertMessage from '@/components/AlertMessage.vue'

export default defineComponent({
  name: 'AccountCreationPage',
  components: {
    AlertMessage
  },
  setup () {
    const error = reactive({
      exists: false,
      message: ''
    })
    const message = ref('')
    const router = useRouter()
    onMounted(async () => {
      const query = router.currentRoute.value.query
      const tempAccountId = query['temp-account-id'] as string | null | undefined
      if (!tempAccountId) {
        error.exists = true
        error.message = Message.INVALID_QUERY_PARAM
        return
      }
      try {
        const result = await createAccount(tempAccountId)
        if (result instanceof CreateAccountResp) {
          error.exists = false
          message.value = Message.ACCOUNT_CREATED
        } else if (result instanceof ApiErrorResp) {
          error.exists = true
          error.message = createErrorMessage(result.getApiError().getCode())
        } else {
          throw new Error(`unexpected result: ${result}`)
        }
      } catch (e) {
        error.exists = true
        error.message = `${Message.ACCOUNT_CREATION_FAILED}: ${e}`
      }
    })
    return { error, message }
  }
})
</script>
