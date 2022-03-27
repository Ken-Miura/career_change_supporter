<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main>
      <div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <button data-test="move-to-create-identity-request-list-page-button" v-on:click="moveToCreateIdentityRequestListPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">本人確認依頼（新規）</button>
          <button data-test="move-to-update-identity-request-list-page-button" v-on:click="moveToUpdateIdentityRequestListPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">本人確認依頼（更新）</button>
          <button data-test="move-to-create-career-request-list-page-button" v-on:click="moveToCreateCareerRequestListPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">職務経歴確認依頼（新規）</button>
          <button data-test="move-to-update-career-request-list-page-button" v-on:click="moveToUpdateCareerRequestListPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">職務経歴確認依頼（更新）</button>
          <button data-test="move-to-disable-account-page-button" v-on:click="moveToDisableAccountPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">アカウント無効化</button>
          <button data-test="move-to-enable-account-page-button" v-on:click="moveToEnableAccountPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">アカウント有効化</button>
          <button data-test="move-to-refund-page-button" v-on:click="moveToRefundPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">返金処理</button>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'

export default defineComponent({
  name: 'AdminMenuPage',
  components: {
    TheHeader
  },
  setup () {
    const router = useRouter()
    const moveToCreateIdentityRequestListPage = async () => {
      await router.push('/create-identity-request-list')
    }
    const moveToUpdateIdentityRequestListPage = async () => {
      await router.push('/update-identity-request-list')
    }
    const moveToCreateCareerRequestListPage = async () => {
      await router.push('/create-career-request-list')
    }
    const moveToUpdateCareerRequestListPage = async () => {
      await router.push('/update-career-request-list')
    }
    const moveToDisableAccountPage = async () => {
      await router.push('/disable-account')
    }
    const moveToEnableAccountPage = async () => {
      await router.push('/enable-account')
    }
    const moveToRefundPage = async () => {
      await router.push('/refund')
    }
    return {
      moveToCreateIdentityRequestListPage,
      moveToUpdateIdentityRequestListPage,
      moveToCreateCareerRequestListPage,
      moveToUpdateCareerRequestListPage,
      moveToDisableAccountPage,
      moveToEnableAccountPage,
      moveToRefundPage
    }
  }
})
</script>
