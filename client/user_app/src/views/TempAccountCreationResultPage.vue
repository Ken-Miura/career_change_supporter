<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-xl lg:text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="bg-white max-w-lg mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
      <div class="flex flex-col justify-center">
        <p data-test="title" class="font-bold text-lg lg:text-2xl">まだ新規登録は完了していません</p>
        <p data-test="message" class="mt-4 font-bold text-base lg:text-lg">
          指定されたメールアドレスにメールを送信しました。メールに記載されたURLをクリックし、新規登録を完了させて下さい。
        </p>
        <div class="mt-4" role="note">
          <div data-test="note-title" class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">メールが届かない場合</div>
          <div data-test="note-message" class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black">
            下記の項目についてご確認下さい。
            <ul class="list-disc list-inside">
              <li>本サイトのドメインのメールの受信が許可されているかどうか</li>
              <li>迷惑メールに振り分けられているかどうか、もしくはゴミ箱に入っているかどうか</li>
              <li>URL付きのメール受信を許可しているかどうか</li>
            </ul>
          </div>
        </div>
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
  name: 'TempAccountCreationResultPage'
})
</script>
