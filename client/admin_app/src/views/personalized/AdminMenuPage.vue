<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main>
      <div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <button data-test="move-to-identity-page-button" v-on:click="test" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">dummy</button>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <button data-test="move-to-identity-page-button" v-on:click="test" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">dummy</button>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'

export default defineComponent({
  name: 'AdminMenuPage',
  components: {
    TheHeader
  },
  setup () {
    const router = useRouter()
    onMounted(async () => {
      console.log('onMounted')
    })
    const test = async () => {
      console.log('test')
      const params = { page: '0', per_page: '50' }
      const query = new URLSearchParams(params)
      const response = await fetch(`/admin/api/create-identity-requests?${query}`, {
        method: 'GET'
      })
      const data = await response.json()
      console.log(data)
    }
    return {
      test
    }
  }
})
</script>
