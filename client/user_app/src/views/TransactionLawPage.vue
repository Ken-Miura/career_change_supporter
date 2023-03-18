<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="bg-white max-w-3xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <!-- TODO: 内容の更新 -->
      <h1 class="font-bold text-2xl">特定商取引法に基づく表記</h1>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-xl">事業者</h2>
        <p class="ml-2 text-lg">XXX株式会社</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-xl">事業者の所在</h2>
        <p class="ml-2 text-lg">〒111-1111</p>
        <p class="ml-2 text-lg">東京都XXX市XXX</p>
        <p class="ml-2 text-lg">電話番号については「お問い合わせ先」にてご請求をいただければ、遅滞なく開示いたします。</p>
        <p class="ml-2 text-lg">※ お問い合わせは下記のメールアドレスからのみ受け付けています。</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-xl">お問い合わせ先</h2>
        <p class="ml-2 text-lg">xxx.test@example.com</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-xl">相談料と手数料</h2>
        <p class="ml-2 text-lg">相談料は消費税を含む価格で表示されています。相談料は、相談を受け付けるユーザー（コンサルタント）の詳細に記載されています。コンサルタントに対して、以下の手数料が課されます。</p>
        <ol class="list-disc ml-8 text-lg">
          <li>プラットフォーム利用料：相談１回につき、相談料の{{ PLATFORM_FEE_IN_PERCENTAGE }}パーセント</li>
          <li>報酬の入金口座への振込手数料：{{ TRANSFER_FEE_IN_YEN }}円</li>
        </ol>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-xl">お支払い方法</h2>
        <p class="ml-2 text-lg">クレジットカード（Visa/Master/American Express/JCB/Diners Club/Discover）</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-xl">個人情報の取扱について</h2>
        <p class="ml-2 text-lg"><router-link class="no-underline hover:underline text-gray-800 hover:text-gray-500" to="/privacy-policy">プライバシーポリシー</router-link>をご参照下さい。</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-xl">返金、キャンセルについて</h2>
        <p class="ml-2 text-lg">サービス特性上、決済後の返金・キャンセルはお受けできません。ただし、サービス提供者側の都合やなんらかの事情で当日のサービス実施が中止になった場合は、全額返金いたします。</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-xl">サービスの提供時期</h2>
        <p class="ml-2 text-lg">相談を申し込みたい方と相談を受け付けたい方が合意した開始時刻に相談を行うことが可能になります。</p>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { TRANSFER_FEE_IN_YEN } from '@/util/personalized/reward/TransferFee'
import { PLATFORM_FEE_IN_PERCENTAGE } from '@/util/personalized/profile/PlatformFee'

export default defineComponent({
  name: 'TransactionLawPage',
  setup () {
    return {
      PLATFORM_FEE_IN_PERCENTAGE,
      TRANSFER_FEE_IN_YEN
    }
  }
})
</script>
