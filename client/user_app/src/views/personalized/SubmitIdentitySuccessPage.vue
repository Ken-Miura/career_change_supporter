<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">{{ message }}</h3>
    </main>
    <footer class="max-w-lg mx-auto flex flex-col text-white">
      <router-link to="/profile" class="hover:underline text-center">プロフィールへ</router-link>
      <router-link to="/" class="mt-6 hover:underline text-center">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import { Message } from '@/util/Message'

export default defineComponent({
  name: 'SubmitIdentitySuccessPage',
  components: {
    TheHeader
  },
  setup () {
    // postIdentityが成功した結果を表示するだけなので、
    // onMounted内でrefreshを呼ぶことはしない
    const message = Message.SUBMIT_IDENTITY_SUCCESS_MESSAGE
    return { message }
  }
})
</script>
