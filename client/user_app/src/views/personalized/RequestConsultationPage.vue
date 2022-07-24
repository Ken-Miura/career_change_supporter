<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">検索</h3>
        <p class="mt-2 text-lg">相談を申し込みたい相手の条件を入力して検索して下さい。</p>
        <form @submit.prevent="moveToConsultantList">
          <div class="m-4 text-2xl grid grid-cols-6">
            <div data-test="company-name-label" class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              勤務先名称（例 xxx株式会社）
            </div>
            <div data-test="company-name-input" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.companyName" v-on:input="setCompanyName" type="text" minlength="0" maxlength="256" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="department-name-label" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              部署名
            </div>
            <div data-test="department-name-input" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.departmentName" v-on:input="setDepartmentName" type="text" minlength="0" maxlength="256" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="office-label" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              勤務地（例 xxx事業所）
            </div>
            <div data-test="office-input" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.office" v-on:input="setOffice" type="text" minlength="0" maxlength="256" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="years-of-service-label" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              在籍年数
            </div>
            <div data-test="years-of-service-select" class="mt-2 w-full text-2xl justify-self-start col-span-6">
              <select v-model="form.yearsOfService" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option value="">指定なし</option>
                <option value="THREE_YEARS_OR_MORE">３年以上</option>
                <option value="FIVE_YEARS_OR_MORE">５年以上</option>
                <option value="TEN_YEARS_OR_MORE">１０年以上</option>
                <option value="FIFTEEN_YEARS_OR_MORE">１５年以上</option>
                <option value="TWENTY_YEARS_OR_MORE">２０年以上</option>
              </select>
            </div>
            <div data-test="employed-label" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              在籍の有無
            </div>
            <div data-test="employed-select" class="mt-2 w-full text-2xl justify-self-start col-span-6">
              <select v-model="form.employed" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option value="">指定なし</option>
                <option value="true">在籍中</option>
                <option value="false">退職済</option>
              </select>
            </div>
            <div data-test="contract-type-label" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              雇用形態
            </div>
            <div data-test="contract-type-select" class="mt-2 w-full text-2xl justify-self-start col-span-6">
              <select v-model="form.contractType" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option value="">指定なし</option>
                <option value="regular">正社員</option>
                <option value="contract">契約社員</option>
                <option value="other">その他</option>
              </select>
            </div>
            <div data-test="profession-label" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              職種（例 ITエンジニア）
            </div>
            <div data-test="profession-input" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.profession" v-on:input="setProfession" type="text" minlength="0" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="annual-income-in-man-yen-label" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              年収（単位：万円）
            </div>
            <div data-test="annual-income-in-man-yen-equal-or-more-input" class="mt-2 min-w-full justify-self-start col-span-5 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.equalOrMoreAnnualIncomeInManYen" v-on:input="setEqualOrMoreAnnualIncomeInManYen" type="text" minlength="0" maxlength="5" pattern="\d*" title="半角数字でご入力下さい。" class="text-right bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="annual-income-in-man-yen-equal-or-more-label" class="ml-2 mt-2 text-2xl justify-self-start col-span-1 pt-3">
              万円以上
            </div>
            <div data-test="annual-income-in-man-yen-equal-or-less-input" class="mt-2 min-w-full justify-self-start col-span-5 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.equalOrLessAnnualIncomeInManYen" v-on:input="setEqualOrLessAnnualIncomeInManYen" type="text" minlength="0" maxlength="5" pattern="\d*" title="半角数字でご入力下さい。" class="text-right bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="annual-income-in-man-yen-equal-or-less-label" class="ml-2 mt-2 text-2xl justify-self-start col-span-1 pt-3">
              万円以下
            </div>
            <div data-test="is-manager-label" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              管理職区分
            </div>
            <div data-test="is-manager-select" class="mt-2 w-full text-2xl justify-self-start col-span-6">
              <select v-model="form.isManager" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option value="">指定なし</option>
                <option value="false">非管理職</option>
                <option value="true">管理職</option>
              </select>
            </div>
            <div data-test="position-name-label" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              職位（例 係長）
            </div>
            <div data-test="position-name-input" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.positionName" v-on:input="setPositionName" type="text" minlength="0" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="is-new-graduate-label" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              入社区分
            </div>
            <div data-test="is-new-graduate-select" class="mt-2 w-full text-2xl justify-self-start col-span-6">
              <select v-model="form.isNewGraduate" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option value="">指定なし</option>
                <option value="true">新卒入社</option>
                <option value="false">中途入社</option>
              </select>
            </div>
            <div data-test="note-label" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              備考
            </div>
            <div data-test="note-input" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <textarea v-bind:value="form.note" v-on:input="setNote" minlength="0" maxlength="2048" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3"></textarea>
            </div>
            <div data-test="fee-per-hour-in-yen-label" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              相談一回（１時間）の相談料（単位：円）
            </div>
            <div data-test="fee-per-hour-in-yen-input" class="mt-2 min-w-full justify-self-start col-span-5 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.equalOrMoreFeePerHourInYen" v-on:input="setEqualOrMoreFeePerHourInYen" type="text" minlength="0" maxlength="5" pattern="\d*" title="半角数字でご入力下さい。" class="text-right bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="fee-per-hour-in-yen-label" class="ml-2 mt-2 text-2xl justify-self-start col-span-1 pt-3">
              円以上
            </div>
            <div data-test="fee-per-hour-in-yen-input" class="mt-2 min-w-full justify-self-start col-span-5 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.equalOrLessFeePerHourInYen" v-on:input="setEqualOrLessFeePerHourInYen" type="text" minlength="0" maxlength="5" pattern="\d*" title="半角数字でご入力下さい。" class="text-right bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="fee-per-hour-in-yen-label" class="ml-2 mt-2 text-2xl justify-self-start col-span-1 pt-3">
              円以下
            </div>
          </div>
          <button data-test="submit-button" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200" type="submit">検索する</button>
          <AlertMessage v-bind:class="['mt-6', { 'hidden': !error.exists }]" v-bind:message="error.message"/>
        </form>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, reactive } from 'vue'
import { useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { useConsultantSearchParam } from './useConsultantSearchParam'
import { Message } from '@/util/Message'

export default defineComponent({
  name: 'RequestConsultationPage',
  components: {
    TheHeader,
    AlertMessage
  },
  setup () {
    const error = reactive({
      exists: false,
      message: ''
    })
    const {
      form,
      setCompanyName,
      setDepartmentName,
      setOffice,
      setProfession,
      setEqualOrMoreAnnualIncomeInManYen,
      setEqualOrLessAnnualIncomeInManYen,
      setPositionName,
      setNote,
      setEqualOrMoreFeePerHourInYen,
      setEqualOrLessFeePerHourInYen
    } = useConsultantSearchParam()
    const router = useRouter()
    onMounted(async () => {
      try {
        const resp = await refresh()
        if (!(resp instanceof RefreshResp)) {
          if (!(resp instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${resp}`)
          }
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          }
          error.exists = true
          error.message = createErrorMessage(resp.getApiError().getCode())
        }
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const moveToConsultantList = async () => {
      console.log('moveToConsultantList')
    }

    return {
      error,
      form,
      setCompanyName,
      setDepartmentName,
      setOffice,
      setProfession,
      setEqualOrMoreAnnualIncomeInManYen,
      setEqualOrLessAnnualIncomeInManYen,
      setPositionName,
      setNote,
      setEqualOrMoreFeePerHourInYen,
      setEqualOrLessFeePerHourInYen,
      moveToConsultantList
    }
  }
})
</script>
