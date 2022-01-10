<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">{{ message }}</h3>
      <div v-if="careers.length !== 0" class="font-bold text-lg">
        <div>props: {{ careerId }}</div>
        <div>{{ careers }}</div>
      </div>
      <p v-else class="font-bold text-lg">careers.length === 0</p>
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
import { Career } from '@/util/profile/Career'

export default defineComponent({
  name: 'EditCareerPage',
  components: {
    TheHeader
  },
  props: {
    careerId: Number
  },
  setup () {
    const message = ref('相談料編集用テストページ')
    const careers = ref([] as Career[])
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
      careers.value = store.state.careers
    })
    return { message, careers }
  }
})
</script>
