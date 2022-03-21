<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main>
      <div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <div class="mt-4 bg-white px-4 py-3 text-black text-xl grid grid-cols-4">
            <div class="mt-2 justify-self-start col-span-2">依頼時刻</div>
            <div class="mt-2 justify-self-start col-span-1">名前</div>
            <div class="mt-2 justify-self-start col-span-1"></div>
          </div>
          <ul>
            <li v-for="item in items" v-bind:key="item">
              <div class="mt-4">
                <div class="border border-gray-600 rounded bg-white px-4 py-3 text-black text-xl grid grid-cols-4">
                  <div class="mt-3 justify-self-start col-span-2">{{ item.requested_at.getFullYear() }}年{{ (item.requested_at.getMonth() + 1).toString().padStart(2, '0') }}月{{ item.requested_at.getDate().toString().padStart(2, '0') }}日{{ item.requested_at.getHours().toString().padStart(2, '0') }}時{{ item.requested_at.getMinutes().toString().padStart(2, '0') }}分{{ item.requested_at.getSeconds().toString().padStart(2, '0') }}秒</div>
                  <div class="mt-3 justify-self-start col-span-1">{{ item.name }}</div>
                  <button class="col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200" v-on:click="test(item.account_id)">詳細を確認する</button>
                </div>
              </div>
            </li>
          </ul>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import { CreateIdentityRequestItem } from '@/util/personalized/create-identity-request-list/CreateIdentityRequestItem'

export default defineComponent({
  name: 'CreateIdentityRequestListPage',
  components: {
    TheHeader
  },
  setup () {
    const items = ref([] as CreateIdentityRequestItem[])
    const router = useRouter()
    onMounted(async () => {
      const params = { page: '0', per_page: '50' }
      const query = new URLSearchParams(params)
      const response = await fetch(`/admin/api/create-identity-requests?${query}`, {
        method: 'GET'
      })
      const reqItems = await response.json() as CreateIdentityRequestItem[]
      reqItems.forEach(e => {
        const utcTime = e.requested_at
        e.requested_at = new Date(utcTime.toLocaleString())
      })
      console.log(reqItems)
      items.value = reqItems
    })
    const test = async (accountId: number) => {
      console.log(accountId)
    }
    return {
      items,
      test
    }
  }
})
</script>
