<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main>
      <div class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-xl">リカバリコード {{ recoveryCode }}</h3>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'EnableMfaSuccessPage',
  components: {
    TheHeader
  },
  setup () {
    const store = useStore()
    const recoveryCode = ref(null as string | null)

    onMounted(async () => {
      recoveryCode.value = store.state.recoveryCode
    })

    return {
      recoveryCode
    }
  }
})
</script>
