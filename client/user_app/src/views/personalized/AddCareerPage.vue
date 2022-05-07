<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="false" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">職務経歴</h3>
        <p class="mt-2 text-lg">相談受け付けを行うために必要となる情報です。職務経歴の確認の依頼後、入力した値が職務経歴に反映された時点で、職務経歴の登録が完了となります。<span class=" text-red-500">相談申し込みの判断に使われるため、他のユーザーに公開されます。</span>入社日と退社日は在籍年数（3年未満、3年以上5年未満、5年以上10年未満、10年以上15年未満、15年以上20年未満、20年以上）という形に変換され、そのまま公開されることはありません。</p>
        <form @submit.prevent="submitCareer">
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              勤務先名称（必須）（例 xxx株式会社）
            </div>
            <div data-test="company-name-div" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input type="text" required minlength="1" maxlength="256" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              部署名（任意）
            </div>
            <div data-test="department-name-div" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input type="text" minlength="1" maxlength="256" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              勤務地（任意）（例 xxx事業所）
            </div>
            <div data-test="office-div" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input type="text" minlength="1" maxlength="256" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              入社日（必須）
            </div>
            <div data-test="career-start-year-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in careerStartYearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div data-test="career-start-year-div" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">年</div>
            <div data-test="career-start-month-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in careerStartMonthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div data-test="career-start-month-div" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">月</div>
            <div data-test="career-start-day-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in careerStartDayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div data-test="career-start-day-div" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">日</div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              退社日（任意）
            </div>
            <div data-test="career-end-year-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in careerEndYearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div data-test="career-end-year-div" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">年</div>
            <div data-test="career-end-month-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in careerEndMonthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div data-test="career-end-month-div" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">月</div>
            <div data-test="career-end-day-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in careerEndDayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div data-test="career-end-day-div" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">日</div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              雇用形態（必須）
            </div>
            <div data-test="contract-type-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-6">
              <select class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option value="regular">正社員</option>
                <option value="contract">契約社員</option>
                <option value="other">その他</option>
              </select>
            </div>
          </div>
          <button data-test="submit-button" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">職務経歴の確認を依頼する</button>
          <AlertMessage v-bind:class="['mt-6', { 'hidden': isHidden }]" v-bind:message="errorMessage"/>
        </form>
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
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'

export default defineComponent({
  name: 'AddCareerPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()
    const isHidden = ref(true)
    const errorMessage = ref('')
    const careerStartYearList = ref([] as string[])
    const careerStartMonthList = ref([] as string[])
    const careerStartDayList = ref([] as string[])
    const careerEndYearList = ref([] as string[])
    const careerEndMonthList = ref([] as string[])
    const careerEndDayList = ref([] as string[])
    onMounted(async () => {
      try {
        const resp = await refresh()
        if (resp instanceof RefreshResp) {
          // セッションが存在し、利用規約に同意済のため、ログイン後のページを表示可能
          // TODO: 正常系の処理
        } else if (resp instanceof ApiErrorResp) {
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          }
          // TODO: エラー処理
        }
      } catch (e) {
        // TODO: エラー処理
      }
      console.log('TODO: 実装後削除')
    })
    const submitCareer = async () => {
      console.log('submitCareer')
    }
    return {
      submitCareer,
      isHidden,
      errorMessage,
      careerStartYearList,
      careerStartMonthList,
      careerStartDayList,
      careerEndYearList,
      careerEndMonthList,
      careerEndDayList
    }
  }
})
</script>
