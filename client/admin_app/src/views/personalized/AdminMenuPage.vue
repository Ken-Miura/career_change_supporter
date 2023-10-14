<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <main>
      <div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
          <button data-test="move-to-create-identity-request-list-page-button" v-on:click="moveToCreateIdentityRequestListPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">本人確認依頼（新規）</button>
          <button data-test="move-to-update-identity-request-list-page-button" v-on:click="moveToUpdateIdentityRequestListPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">本人確認依頼（更新）</button>
          <button data-test="move-to-create-career-request-list-page-button" v-on:click="moveToCreateCareerRequestListPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">職務経歴確認依頼</button>
          <button data-test="move-to-user-account-search-page-button" v-on:click="moveToUserAccountSearchPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">アカウント情報確認</button>
          <button v-on:click="moveToAwaitingPaymentsPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">入金待ちリスト</button>
          <button v-on:click="moveToExpiredAwaitingPaymentsPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">期限切れ入金待ちリスト</button>
          <button v-on:click="moveToNeglectedPaymentsPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">無視された入金リスト</button>
          <button v-on:click="moveToAwaitingWithdrawalsPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">出金待ちリスト</button>
          <button v-on:click="moveToReceiptsOfConsultationPagePage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">領収書リスト</button>
          <button v-on:click="moveToRefundedPaymentsPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">返金済みリスト</button>
          <button v-on:click="moveToLeftAwaitingWithdrawalsPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">放置された報酬リスト</button>
          <button v-on:click="moveToMaintenancesPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">メンテナンス期間設定</button>
          <button v-on:click="moveToNewsPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">お知らせ作成</button>
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

    const moveToUserAccountSearchPage = async () => {
      await router.push('/user-account-search')
    }

    const moveToAwaitingPaymentsPage = async () => {
      await router.push('/awaiting-payments?page=0&per-page=20')
    }

    const moveToExpiredAwaitingPaymentsPage = async () => {
      await router.push('/expired-awaiting-payments?page=0&per-page=20')
    }

    const moveToNeglectedPaymentsPage = async () => {
      await router.push('/neglected-payments?page=0&per-page=20')
    }

    const moveToAwaitingWithdrawalsPage = async () => {
      await router.push('/awaiting-withdrawals?page=0&per-page=20')
    }

    const moveToReceiptsOfConsultationPagePage = async () => {
      await router.push('/receipts-of-consultation?page=0&per-page=20')
    }

    const moveToRefundedPaymentsPage = async () => {
      await router.push('/refunded-payments?page=0&per-page=20')
    }

    const moveToLeftAwaitingWithdrawalsPage = async () => {
      await router.push('/left-awaiting-withdrawals?page=0&per-page=20')
    }

    const moveToMaintenancesPage = async () => {
      await router.push('/maintenances')
    }

    const moveToNewsPage = async () => {
      await router.push('/news')
    }

    return {
      moveToCreateIdentityRequestListPage,
      moveToUpdateIdentityRequestListPage,
      moveToCreateCareerRequestListPage,
      moveToUserAccountSearchPage,
      moveToAwaitingPaymentsPage,
      moveToExpiredAwaitingPaymentsPage,
      moveToAwaitingWithdrawalsPage,
      moveToNeglectedPaymentsPage,
      moveToReceiptsOfConsultationPagePage,
      moveToRefundedPaymentsPage,
      moveToLeftAwaitingWithdrawalsPage,
      moveToMaintenancesPage,
      moveToNewsPage
    }
  }
})
</script>
