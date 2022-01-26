<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">ユーザー情報</h3>
        <p class="mt-2 text-lg">本人確認のために利用される情報です（本人確認の完了後、相談申し込みが可能となります）本人確認の依頼後、入力した値がユーザー情報に反映された時点で、本人確認が完了となります。ユーザー情報が他のユーザーに公開されることはありません。</p>
        <form @submit.prevent="submitIdentity">
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              氏名
            </div>
            <div class="mt-2 justify-self-start col-span-3 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">姓</label>
              <input v-bind:value="form.lastName" v-on:input="setLastName" type="text" required minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 ml-1 justify-self-start col-span-3 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">名</label>
              <input v-bind:value="form.firstName" v-on:input="setFirstName" type="text" required minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              フリガナ
            </div>
            <div class="mt-2 justify-self-start col-span-3 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">セイ</label>
              <input v-bind:value="form.lastNameFurigana" v-on:input="setLastNameFurigana" type="text" required minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 ml-1 justify-self-start col-span-3 pt-3 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">メイ</label>
              <input v-bind:value="form.firstNameFurigana" v-on:input="setFirstNameFurigana" type="text" required minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              生年月日
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="form.yearOfBirth" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in yearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              年
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="form.monthOfBirth" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in monthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              月
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="form.dayOfBirth" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in dayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              日
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              住所
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-6">
              <select v-model="form.prefecture" class="block w-full px-3 py-6 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="prefecture in prefectureList" v-bind:key="prefecture" v-bind:value="prefecture">{{ prefecture }}</option>
              </select>
            </div>
            <div class="mt-2 w-full justify-self-start col-span-6 pt-3 pl-2 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">市区町村</label>
              <input v-bind:value="form.city" v-on:input="setCity" type="text" required minlength="1" maxlength="32" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 w-full justify-self-start col-span-6 pt-3 pl-2 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">番地</label>
              <input v-bind:value="form.addressLine1" v-on:input="setAddressLine1" type="text" required minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-2 w-full justify-self-start col-span-6 pt-3 pl-2 rounded bg-gray-200">
              <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">建物名・部屋番号</label>
              <input v-bind:value="form.addressLine2" v-on:input="setAddressLine2" type="text" minlength="0" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              電話番号
            </div>
            <div class="mt-2 w-full justify-self-start col-span-6 pt-3 pl-2 rounded bg-gray-200">
              <input v-bind:value="form.telephoneNumber" v-on:input="setTelephoneNumber" type="text" inputmode="tel" pattern="\d*" title="半角数字以外が入力されています。半角数字を用いてご入力下さい。" required minlength="10" maxlength="13" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              身分証明書
            </div>
            <div class="mt-2 text-xl justify-self-start col-span-6 pt-3 pl-3">
              身分証明書の画像は<span class=" text-red-500">pngまたはjpegで、サイズが7MB以下</span>である必要が有ります。<span class=" text-red-500">運転免許証、マイナンバーカードまたはパスポート</span>を身分証明書としてご利用可能です。運転免許証は表面と裏面、マイナンバーカードは表面（顔写真記載面）のみ、パスポートは顔写真記載面と現住所記載面をアップロードしてください（いずれも<span class=" text-red-500">有効期限内</span>のものをアップロードください）運転免許証は<span class=" text-red-500">臓器提供に関する意思（裏面下半分）</span>、マイナンバーカードは<span class=" text-red-500">性別、臓器提供意思</span>、パスポートは<span class=" text-red-500">性別、本籍</span>を<span class=" text-red-500">マスキング</span>した状態でアップロードして下さい）
            </div>
            <div class="mt-6 pl-3 w-full justify-self-start col-span-1">
              表面
            </div>
            <div class="mt-2 w-full justify-self-start col-span-5 pt-3 rounded bg-gray-200">
              <input type="file" name="file1" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-6 pl-3 w-full justify-self-start col-span-1">
              裏面
            </div>
            <div class="mt-2 w-full justify-self-start col-span-5 pt-3 rounded bg-gray-200">
              <input type="file" name="file2" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
          </div>
          <button class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">本人確認を依頼する</button>
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
import { useStore } from 'vuex'
import { useIdentity } from '@/views/personalized/useIdentity'
import TheHeader from '@/components/TheHeader.vue'
import { useRouter } from 'vue-router'
import AlertMessage from '@/components/AlertMessage.vue'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { Message } from '@/util/Message'
import { createPrefectureList } from '@/util/personalized/profile/PrefectureList'
import { createDayList } from '@/util/DayList'
import { createMonthList } from '@/util/MonthList'
import { createYearOfBirthList, MIN_AGE, START_YEAR } from '@/util/personalized/profile/YearOfBirthList'

export default defineComponent({
  name: 'IdentityPage',
  components: {
    TheHeader,
    AlertMessage
  },
  setup () {
    const router = useRouter()
    const store = useStore()
    const isHidden = ref(true)
    const errorMessage = ref('')
    const {
      form,
      setLastName,
      setFirstName,
      setLastNameFurigana,
      setFirstNameFurigana,
      setCity,
      setAddressLine1,
      setAddressLine2,
      setTelephoneNumber
    } = useIdentity()
    const currentYear = new Date().getFullYear()
    const yearList = ref(createYearOfBirthList(START_YEAR, currentYear, MIN_AGE))
    const monthList = ref(createMonthList())
    const dayList = ref(createDayList())
    const prefectureList = ref(createPrefectureList())
    onMounted(async () => {
      try {
        const resp = await refresh()
        if (resp instanceof RefreshResp) {
          // セッションが存在し、利用規約に同意済のため、ログイン後のページを表示可能
          // 表示する際の初期値として使いたいだけなので、identityはrefとして宣言しない（リアクティブとしない）
          const identity = store.state.identity
          if (identity !== null) {
            /* eslint-disable camelcase */
            form.lastName = identity.last_name
            form.firstName = identity.first_name
            form.lastNameFurigana = identity.last_name_furigana
            form.firstNameFurigana = identity.first_name_furigana
            form.yearOfBirth = identity.date_of_birth.year
            form.monthOfBirth = identity.date_of_birth.month
            form.dayOfBirth = identity.date_of_birth.day
            form.prefecture = identity.prefecture
            form.city = identity.city
            form.addressLine1 = identity.address_line1
            if (identity.address_line2 !== null) {
              form.addressLine2 = identity.address_line2
            }
            form.telephoneNumber = identity.telephone_number
            /* eslint-enable camelcase */
          }
        } else if (resp instanceof ApiErrorResp) {
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('terms-of-use')
            return
          }
          throw new Error(`unexpected result: ${resp}`)
        } else {
          throw new Error(`unexpected result: ${resp}`)
        }
      } catch (e) {
        isHidden.value = false
        errorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })
    const submitIdentity = async () => {
      console.log(form.lastName)
      console.log(form.firstName)
      console.log(form.lastNameFurigana)
      console.log(form.firstNameFurigana)
      console.log(form.yearOfBirth)
      console.log(form.monthOfBirth)
      console.log(form.dayOfBirth)
      console.log(form.prefecture)
      console.log(form.city)
      console.log(form.addressLine1)
      console.log(form.addressLine2)
      console.log(form.telephoneNumber)
    }
    return {
      isHidden,
      errorMessage,
      form,
      setLastName,
      setFirstName,
      setLastNameFurigana,
      setFirstNameFurigana,
      setCity,
      setAddressLine1,
      setAddressLine2,
      setTelephoneNumber,
      yearList,
      monthList,
      dayList,
      prefectureList,
      submitIdentity
    }
  }
})
</script>
