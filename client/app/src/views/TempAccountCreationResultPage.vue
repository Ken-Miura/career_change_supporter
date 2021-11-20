<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 bo min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="flex justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 v-if="emailAddress != undefined" class="font-bold text-lg">{{ emailAddress }}宛にメールを送信しました。メールを確認し、新規登録を完了させて下さい（メールが届いていない場合、迷惑メールに振り分けられていないか、もしくは本サイトのドメインのメールの受信が許可されているかご確認下さい）</h3>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'TempAccountCreationResultPage',
  props: {
    emailAddress: String
  }
})
</script>
