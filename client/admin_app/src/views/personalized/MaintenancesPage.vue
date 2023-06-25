<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 data-test="label" class="font-bold text-2xl">メンテナンス</h3>
      <p data-test="description" class="ml-2 mt-4 mb-6 text-lg">あああ</p>
    </main>
    <footer class="max-w-lg mx-auto flex flex-col text-white">
      <router-link to="/admin-menu" class="hover:underline text-center">管理メニューへ</router-link>
      <router-link to="/" class="mt-6 hover:underline text-center">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import { useRouter } from 'vue-router'

export default defineComponent({
  name: 'MaintenancesPage',
  components: {
    TheHeader
  },
  setup () {
    const router = useRouter()

    return {
    }
  }
})
</script>
