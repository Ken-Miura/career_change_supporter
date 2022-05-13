<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="bg-white max-w-3xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <!-- TODO: 内容の更新 -->
      <h1 class="font-bold text-2xl">特定商取引法に基づく表記</h1>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-xl">事業者</h2>
        <p class="ml-2 text-lg">XXX株式会社</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-xl">事業者の所在</h2>
        <p class="ml-2 text-lg">XXX株式会社</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-xl">お問い合わせ先</h2>
        <p class="ml-2 text-lg">xxx.test@example.com</p>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'TransactionLawPage'
})
</script>
