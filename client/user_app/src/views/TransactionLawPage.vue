<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-xl lg:text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="bg-white max-w-3xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
      <h1 class="font-bold text-xl lg:text-2xl">特定商取引法に基づく表記</h1>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-lg lg:text-xl">事業者名</h2>
        <p class="ml-2 text-base lg:text-lg">{{ businessOperatorName }}</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-lg lg:text-xl">事業者の所在</h2>
        <p class="ml-2 text-base lg:text-lg">〒{{ businessOperatorZipCode }}</p>
        <p class="ml-2 text-base lg:text-lg">{{ businessOperatorAddress }}</p>
        <p class="ml-2 text-base lg:text-lg">電話番号: {{ businessOperatorPhoneNumber }}</p>
        <p class="ml-2 text-base lg:text-lg">※ お問い合わせは下記のメールアドレスからのみ受け付けています。</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-lg lg:text-xl">お問い合わせ先</h2>
        <p class="ml-2 text-base lg:text-lg">{{ businessOperatorInquiryEmailAddress }}</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-lg lg:text-xl">相談料と手数料</h2>
        <p class="ml-2 text-base lg:text-lg">相談料は消費税を含む価格で表示されています。相談料は、相談を受け付けるユーザー（コンサルタント）の詳細に記載されています。コンサルタントに対して、以下の手数料が課されます。</p>
        <ol class="list-disc ml-8 text-base lg:text-lg">
          <li>プラットフォーム利用料：相談１回につき、相談料の{{ platformFeeInPercentage }}パーセント</li>
          <li>振込手数料：報酬の振込の際に{{ transferFeeInYen }}円</li>
        </ol>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-lg lg:text-xl">お支払い方法</h2>
        <p class="ml-2 text-base lg:text-lg">クレジットカード（Visa/Master/American Express/JCB/Diners Club/Discover）</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-lg lg:text-xl">個人情報の取扱について</h2>
        <p class="ml-2 text-base lg:text-lg"><router-link class="no-underline hover:underline text-gray-800 hover:text-gray-500" to="/privacy-policy">プライバシーポリシー</router-link>をご参照下さい。</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-lg lg:text-xl">サービスの提供時期</h2>
        <p class="ml-2 text-base lg:text-lg">相談を申し込みたい方と相談を受け付けたい方が合意した開始時刻に相談を行うことが可能になります。</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-lg lg:text-xl">支払い確定の時期</h2>
        <p class="ml-2 text-base lg:text-lg">下記のいずれかで相談を申し込んだユーザーの相談料の支払いが確定します。</p>
        <ol class="list-disc ml-8 text-base lg:text-lg">
          <li>相談を申し込んだユーザーが相談を受け付けたユーザー（コンサルタント）を評価したとき支払いが確定します（評価は相談終了時刻以降から可能になります）</li>
          <li>相談を申し込んだユーザーが相談終了時刻から7日経過してもコンサルタントの評価を行っていない場合、任意のタイミングで支払いが確定されます</li>
        </ol>
        <div class="mt-4 ml-2">
          <h2 class="font-bold text-lg lg:text-xl">返金、キャンセルについて</h2>
          <p class="ml-2 text-base lg:text-lg">サービスの特性上、決済後の返金・キャンセルはお受けできません。</p>
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

export default defineComponent({
  name: 'TransactionLawPage',
  setup () {
    const businessOperatorName = process.env.VUE_APP_BUSINESS_OPERATOR_NAME
    const businessOperatorZipCode = process.env.VUE_APP_BUSINESS_OPERATOR_ZIP_CODE
    const businessOperatorAddress = process.env.VUE_APP_BUSINESS_OPERATOR_ADDRESS
    const businessOperatorPhoneNumber = process.env.VUE_APP_BUSINESS_OPERATOR_PHONE_NUMBER
    const businessOperatorInquiryEmailAddress = process.env.VUE_APP_BUSINESS_OPERATOR_INQUIRY_EMAIL_ADDRESS
    const platformFeeInPercentage = process.env.VUE_APP_PLATFORM_FEE_IN_PERCENTAGE
    const transferFeeInYen = process.env.VUE_APP_TRANSFER_FEE_IN_YEN

    return {
      businessOperatorName,
      businessOperatorZipCode,
      businessOperatorAddress,
      businessOperatorPhoneNumber,
      businessOperatorInquiryEmailAddress,
      platformFeeInPercentage,
      transferFeeInYen
    }
  }
})
</script>
