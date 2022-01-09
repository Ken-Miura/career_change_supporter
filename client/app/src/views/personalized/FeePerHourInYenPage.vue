<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">{{ message }}</h3>
      <p v-if="fee !== null" class="font-bold text-lg">{{ fee }}</p>
      <p v-else class="font-bold text-lg">fee === null</p>
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
import { useStore } from 'vuex'

export default defineComponent({
  name: 'FeePerHourInYenPage',
  components: {
    TheHeader
  },
  setup () {
    const message = ref('相談料用テストページ')
    const fee = ref(null as number | null)
    const router = useRouter()
    const store = useStore()
    onMounted(async () => {
      const result = await getPageKindToDisplay()
      if (result === 'personalized-page') {
        // 遷移せずにページを表示
      } else if (result === 'login') {
        await router.push('login')
        return
      } else if (result === 'term-of-use') {
        await router.push('terms-of-use')
        return
      } else {
        throw new Error('Assertion Error: must not reach this line')
      }
      fee.value = store.state.feePerHourInYen
    })
    return { message, fee }
  }
})
</script>
