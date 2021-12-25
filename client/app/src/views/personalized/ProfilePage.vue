<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 bo min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-lg">Eメールアドレス</h3>
        <p class="mt-2 text-lg">登録したEメールアドレスです。他のユーザーに公開されることはありません。</p>
        <p class="mt-4 ml-4 text-2xl">{{ emailAddress }}</p>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-lg">ユーザー情報</h3>
        <p class="mt-2 text-lg">身分証明のために入力する情報で、相談申し込みを行うために必要となる情報です。他のユーザーに公開されることはありません。</p>
        <div v-if="identity != null" class="m-4 text-2xl grid grid-cols-2">
          <div class="justify-self-start">名前</div><div class="justify-self-start">{{identity.last_name}} {{identity.first_name}}</div>
          <div class="justify-self-start">電話番号</div><div class="justify-self-start">{{identity.telephone_number}}</div>
          <div class="justify-self-start">test1</div><div class="justify-self-start">あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ</div>
          <div class="justify-self-start">test2</div><div class="justify-self-start break-all">aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa</div>
        </div>
        <p v-else class="m-4 text-xl">ユーザー情報が設定されていません。</p>
        <button v-on:click="TODO" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">ユーザー情報を編集する</button>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-lg">職務経歴</h3>
        <p class="mt-2 text-lg">相談受け付けを行うために必要となる情報です。<span class=" text-red-500">相談申込みの判断に使われるため、他のユーザーに公開されます。</span></p>
        <p class="mt-4 text-lg">職務経歴サンプル</p>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-lg">相談一回（１時間）の相談料</h3>
        <p class="mt-2 text-lg">相談受け付けを行うために必要となる情報です。<span class=" text-red-500">相談申込みの判断に使われるため、他のユーザーに公開されます。</span></p>
        <p v-if="feePerHourInYen != null" class="m-4 text-2xl">{{ feePerHourInYen }}円</p>
        <p v-else class="m-4 text-xl">相談料が設定されていません。</p>
        <button v-on:click="TODO" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">相談料を編集する</button>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-lg">相談料の入金口座</h3>
        <p class="mt-2 text-lg">受け取った相談料を入金するための口座で、相談受け付けを行うために必要となる情報です。他のユーザーに公開されることはありません。ユーザー情報で身分証明が完了した姓名と異なる名義の口座は設定できません。</p>
        <p class="mt-4 text-lg">相談料の入金口座サンプル</p>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-lg">今月の相談料の合計</h3>
        <p class="mt-2 text-lg">今月承諾した相談の相談料の合計です。他のユーザーに公開されることはありません。</p>
        <p class="mt-4 text-lg">サンプル</p>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-lg">入金情報</h3>
        <p class="mt-2 text-lg">受け取った相談料に関する直近二回分の入金情報です。毎月月末に、前月の相談料の合計から振込手数料が差し引かれた金額が入金されます。他のユーザーに公開されることはありません。</p>
        <p class="mt-4 text-lg">サンプル</p>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <button v-on:click="TODO" class="bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">アカウントを削除する</button>
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
import { getPageKindToDisplay } from '@/util/GetPageKindToDisplay'
import TheHeader from '@/components/TheHeader.vue'
import { getProfile } from '@/util/profile/GetProfile'
import { GetProfileResp } from '@/util/profile/GetProfileResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Identity } from '@/util/profile/Identity'

export default defineComponent({
  name: 'ProfilePage',
  components: {
    TheHeader
  },
  setup () {
    const emailAddress = ref('')
    const feePerHourInYen = ref(0 as number | null)
    const identity = ref(null as Identity | null)
    const router = useRouter()
    onMounted(async () => {
      const result = await getPageKindToDisplay()
      if (result === 'personalized-page') {
        // 遷移せずにページを表示
      } else if (result === 'login') {
        router.push('login')
      } else if (result === 'term-of-use') {
        router.push('terms-of-use')
      } else {
        throw new Error('Assertion Error: must not reach this line')
      }
      const response = await getProfile()
      if (response instanceof GetProfileResp) {
        console.log(response.getProfile())
        const profile = response.getProfile()
        /* eslint-disable camelcase */
        emailAddress.value = profile.email_address
        feePerHourInYen.value = profile.fee_per_hour_in_yen
        identity.value = profile.identity
        /* eslint-enable camelcase */
      } else if (response instanceof ApiErrorResp) {
        console.log('ApiErrorResp')
      } else {
        console.log('else')
      }
    })
    return { emailAddress, identity, feePerHourInYen }
  }
})
</script>
