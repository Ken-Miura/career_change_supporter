<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">ユーザー情報</h3>
          <p class="mt-2 text-lg">身分証明のために入力する情報で、相談申し込みを行うために必要となる情報です。他のユーザーに公開されることはありません。</p>
          <div v-if="identity !== null" class="m-4 text-2xl grid grid-cols-3">
            <div class="mt-2 justify-self-start col-span-1">名前</div><div class="justify-self-start col-span-2">{{ identity.last_name }} {{ identity.first_name }}</div>
            <div class="mt-2 justify-self-start col-span-1">フリガナ</div><div class="justify-self-start col-span-2">{{ identity.last_name_furigana }} {{ identity.first_name_furigana }}</div>
            <div class="mt-2 justify-self-start col-span-1">性別</div><div v-if="identity.sex === 'male'" class="justify-self-start col-span-2">男性</div><div v-else class="justify-self-start col-span-2">女性</div>
            <div class="mt-2 justify-self-start col-span-1">生年月日</div><div class="justify-self-start col-span-2">{{ identity.date_of_birth.year }}年{{ identity.date_of_birth.month }}月{{ identity.date_of_birth.day }}日</div>
            <div class="mt-2 justify-self-start col-span-3">住所</div>
            <div class="mt-2 ml-3 justify-self-start col-span-1">都道府県</div><div class="justify-self-start col-span-2">{{ identity.prefecture }}</div>
            <div class="mt-2 ml-3 justify-self-start col-span-1">市区町村</div><div class="justify-self-start col-span-2">{{ identity.city }}</div>
            <div class="mt-2 ml-3 justify-self-start col-span-1">番地</div><div class="justify-self-start col-span-2">{{ identity.address_line1 }}</div>
            <div v-if="identity.address_line2 !== null" class="mt-2 ml-3 justify-self-start col-span-1">建物名・部屋番号</div><div v-if="identity.address_line2 !== null" class="justify-self-start col-span-2">{{ identity.address_line2 }}</div>
            <div class="mt-2 justify-self-start col-span-1">電話番号</div><div class="justify-self-start col-span-2">{{ identity.telephone_number }}</div>
          </div>
          <p v-else class="m-4 text-xl">ユーザー情報が設定されていません。</p>
          <button v-on:click="TODO" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">ユーザー情報を編集する</button>
        </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import { useStore } from 'vuex'
import TheHeader from '@/components/TheHeader.vue'

export default defineComponent({
  name: 'IdentityPage',
  components: {
    TheHeader
  },
  setup () {
    const store = useStore()
    const identity = ref(store.state.identity)
    return { identity }
  }
})
</script>
