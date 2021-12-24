<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 bo min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">{{ message }}</h3>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { getPageKindToDisplay } from '@/util/GetPageKindToDisplay'
import TheHeader from '@/components/TheHeader.vue'

export default defineComponent({
  name: 'RequestConsultationPage',
  components: {
    TheHeader
  },
  setup () {
    const message = ref('相談申し込み用テストページ')
    const router = useRouter()
    onMounted(async () => {
      const result = await getPageKindToDisplay()
      if (result === 'personalized-page') {
        // 遷移せずにページを表示
      } else if (result === 'login') {
        router.push('login')
      } else if (result === 'term-of-use') {
        router.push('terms-of-use')
      } else {
        throw new Error('Assertion Error: must not reach this line')
      }
    })
    return { message }
  }
})
</script>
