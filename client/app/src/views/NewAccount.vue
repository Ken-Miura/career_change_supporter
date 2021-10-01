<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 bo min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <section>
        <h3 class="font-bold text-2xl">新規登録</h3>
      </section>
      <section class="mt-10">
        <form class="flex flex-col" @submit.prevent="createNewAccount">
          <EmailAddress @on-email-address-updated="setEmailAddress"/>
          <Password @on-password-updated="setPassword" label="パスワード"/>
          <Password @on-password-updated="setPasswordConfirmation" label="パスワード（確認）"/>
          <button class="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">新規登録</button>
        </form>
      </section>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import EmailAddress from '@/components/EmailAddress.vue'
import Password from '@/components/Password.vue'
import { useCredentil } from '@/components/useCredential'
import { useRouter } from 'vue-router'

export default defineComponent({
  name: 'NewAccount',
  components: {
    EmailAddress,
    Password
  },
  setup () {
    const router = useRouter()
    const {
      form,
      setEmailAddress,
      setPassword,
      setPasswordConfirmation,
      passwordsAreSame
    } =
    useCredentil()
    const createNewAccount = async () => {
      if (!passwordsAreSame.value) {
        console.error('!passwordsAreSame.value')
        return
      }
      // eslint-disable-next-line
      const data = { email_address: form.emailAddress, password: form.password }
      let response
      try {
        response = await fetch('/api/temp-accounts', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json; charset=utf-8' },
          body: JSON.stringify(data)
        })
        const result = await response.json()
        console.log(result)
        if (response.status === 200) {
          await router.push({ name: 'TempAccountCreated', params: { emailAddress: result.email_address } })
        }
      } catch (e) {
        console.log(`failed to get response: ${e}`)
      }
    }
    return {
      form,
      setEmailAddress,
      setPassword,
      setPasswordConfirmation,
      createNewAccount
    }
  }
})
</script>
