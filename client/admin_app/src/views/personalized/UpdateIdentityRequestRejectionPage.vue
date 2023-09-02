<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="flex flex-col justify-center items-center bg-white max-w-lg mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">本人確認依頼（更新）を拒否しました。</h3>
      <router-link to="/update-identity-request-list" class="m-4 hover:underline">本人確認依頼（更新）一覧へ</router-link>
    </main>
    <footer class="max-w-lg mx-auto flex flex-col text-white">
      <router-link to="/admin-menu" class="hover:underline text-center">管理メニューへ</router-link>
      <router-link to="/" class="mt-6 hover:underline text-center">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
</script>
