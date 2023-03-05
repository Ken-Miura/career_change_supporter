<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg text-center">{{ message }}</h3>
      <router-link to="/unrated-item-list" class="mt-4 text-center hover:underline">未評価のコンサルタント、ユーザー一覧へ</router-link>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import { Message } from '@/util/Message'

export default defineComponent({
  name: 'RateSuccessPage',
  components: {
    TheHeader
  },
  setup () {
    // 評価が成功した結果を表示するだけなので、
    // onMounted内でrefreshを呼ぶことはしない
    const message = Message.RATE_SUCCESS_MESSAGE
    return { message }
  }
})
</script>
