<template>
  <nav class="fixed w-full z-30 top-0 text-white bg-gradient-to-r from-gray-500 to-gray-900" >
    <div class="w-full container mx-auto flex flex-wrap items-center justify-between mt-0 py-2">
      <div class="pl-4 flex items-center">
        <router-link class="toggleColour text-white no-underline hover:no-underline font-bold text-xl lg:text-xl" to="/">トップページ</router-link>
      </div>
      <div class="block lg:hidden pr-4">
        <button v-on:click="switchMenuState" class="flex items-center p-1 text-white focus:outline-none focus:shadow-outline transform transition hover:scale-105 duration-300 ease-in-out">
          <svg class="fill-current h-6 w-6" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
            <title>Menu</title>
            <path d="M0 3h20v2H0V3zm0 6h20v2H0V9zm0 6h20v2H0v-2z" />
          </svg>
        </button>
      </div>
      <div data-test="div" v-bind:class="['w-full', 'flex-grow', 'lg:flex', 'lg:items-center', 'lg:w-auto', { 'hidden': isHidden }, 'mt-2', 'lg:mt-0', 'bg-gray-500', 'lg:bg-transparent', 'text-black', 'p-4', 'lg:p-0 z-20']">
        <ul class="list-reset lg:flex justify-end flex-1 items-center">
          <li class="mr-3">
            <router-link class="inline-block text-white no-underline py-2 px-4" to="/profile">プロフィール</router-link>
          </li>
          <li class="mr-3">
            <router-link class="inline-block text-white no-underline py-2 px-4" to="/reward">報酬</router-link>
          </li>
          <li class="mr-3">
            <router-link class="inline-block text-white no-underline py-2 px-4" to="/consultants-search">相談申し込み</router-link>
          </li>
          <li class="mr-3">
            <router-link class="inline-block text-white no-underline py-2 px-4" to="/consultation-request-list">相談受け付け</router-link>
          </li>
          <li class="mr-3">
            <router-link class="inline-block text-white no-underline py-2 px-4" to="/audio-test">音声入出力テスト</router-link>
          </li>
          <li class="mr-3">
            <router-link class="inline-block text-white no-underline py-2 px-4" to="/schedule">スケジュール</router-link>
          </li>
          <li class="mr-3">
            <router-link class="inline-block text-white no-underline py-2 px-4" to="/awaiting-rating-list">評価</router-link>
          </li>
          <li class="mr-3">
            <p data-test="p" class="inline-block text-white no-underline py-2 px-4 cursor-pointer" v-on:click="logoutHandler">ログアウト</p>
          </li>
        </ul>
      </div>
    </div>
    <hr class="border-b border-gray-100 opacity-25 my-0 py-0" />
  </nav>
</template>

<script lang="ts">
import { logout } from '@/util/personalized/logout/Logout'
import { defineComponent, ref } from 'vue'
import { useRouter } from 'vue-router'

export default defineComponent({
  name: 'TheHeader',
  setup () {
    const isHidden = ref(true)
    const switchMenuState = () => {
      isHidden.value = !isHidden.value
    }
    const router = useRouter()
    const logoutHandler = async () => {
      try {
        // ログアウトの成否がユーザーの関心事になるとは思えないため、戻り値の確認はしない
        await logout()
      } catch (e) {
        // ログアウトが失敗したことがユーザーの関心事になるとは思えないため、無視してログイン画面へ遷移する
        // コネクションエラーの場合、ログイン画面でログイン失敗時にエラーメッセージが表示されるので、ここで表示する必要性もない
      }
      await router.push('/login')
    }
    return {
      isHidden,
      switchMenuState,
      logoutHandler
    }
  }
})
</script>
