<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">{{ message }}</h3>
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
  name: 'PostIdentityResultPage',
  components: {
    TheHeader
  },
  setup () {
    // postIdentityが成功した結果を表示するだけなので、
    // onMounted内でrefreshを呼ぶことはしない
    const store = useStore()
    const message = ref(store.state.postIdentityResultMessage)
    return { message }
  }
})
</script>
