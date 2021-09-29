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
        <form class="flex flex-col" method="POST" action="#">
          <EmailAddress/>
          <Password label="パスワード"/>
          <Password label="パスワード（確認）"/>
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

export default defineComponent({
  name: 'NewAccount',
  components: {
    EmailAddress,
    Password
  }
})
</script>
