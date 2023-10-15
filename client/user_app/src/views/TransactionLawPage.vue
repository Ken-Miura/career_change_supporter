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
        <p class="ml-2 text-base lg:text-lg">電話番号: 電話番号開示を希望される場合はお問い合わせ先よりメールでご連絡いただければ遅延なく開示いたします（※ お問い合わせは下記のメールアドレスからのみ受け付けています）</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-lg lg:text-xl">お問い合わせ先</h2>
        <p class="ml-2 text-base lg:text-lg">{{ businessOperatorInquiryEmailAddress }}</p>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-lg lg:text-xl">相談料と手数料</h2>
        <p class="ml-2 text-base lg:text-lg">相談料は、相談を受け付けるユーザー（コンサルタント）の詳細に記載されています。相談を行ったコンサルタントに対しては、相談料から以下の手数料が差し引かれた金額が、コンサルタントの指定する口座に振り込まれます。</p>
        <ol class="list-disc ml-8 text-base lg:text-lg">
          <li>プラットフォーム利用料：相談１回につき、相談料の{{ platformFeeRateInPercentage }}パーセント</li>
          <li>振込手数料：{{ transferFeeInYen }}円</li>
        </ol>
      </div>
      <div class="mt-4 ml-2">
        <h2 class="font-bold text-lg lg:text-xl">相談料のお支払い方法</h2>
        <p class="ml-2 text-base lg:text-lg">銀行口座振込（相談申し込みがコンサルタントに承認された際、本サービスが管理する口座を振込先として、その口座情報をメールでお知らせ致します）</p>
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
        <h2 class="font-bold text-lg lg:text-xl">相談料支払いの確定時期</h2>
        <p class="ml-2 text-base lg:text-lg">相談終了時刻から{{ waitingPeriodBeforeWithdrawalToConsultantInDays }}日経過後に相談を申し込んだユーザーの相談料の支払いが確定します。相談終了時刻から{{ waitingPeriodBeforeWithdrawalToConsultantInDays }}日経過後以降、相談料から手数料を差し引いた金額がコンサルタントの指定した口座に順次振り込まれます。</p>
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
    const businessOperatorInquiryEmailAddress = process.env.VUE_APP_BUSINESS_OPERATOR_INQUIRY_EMAIL_ADDRESS
    const platformFeeRateInPercentage = process.env.VUE_APP_PLATFORM_FEE_RATE_IN_PERCENTAGE
    const transferFeeInYen = process.env.VUE_APP_TRANSFER_FEE_IN_YEN
    const waitingPeriodBeforeWithdrawalToConsultantInDays = process.env.VUE_APP_WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS

    return {
      businessOperatorName,
      businessOperatorZipCode,
      businessOperatorAddress,
      businessOperatorInquiryEmailAddress,
      platformFeeRateInPercentage,
      transferFeeInYen,
      waitingPeriodBeforeWithdrawalToConsultantInDays
    }
  }
})
</script>
