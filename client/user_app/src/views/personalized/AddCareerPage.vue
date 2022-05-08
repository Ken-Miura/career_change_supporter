<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="waitingRequestDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">職務経歴</h3>
        <p class="mt-2 text-lg">相談受け付けを行うために必要となる情報です。職務経歴の確認を依頼後、入力した値が反映された時点で、職務経歴の登録が完了となります。<span class=" text-red-500">相談申し込みの判断に使われるため、他のユーザーに公開されます。</span>入社日と退社日は在籍年数（3年未満、3年以上5年未満、5年以上10年未満、10年以上15年未満、15年以上20年未満、20年以上）という形に変換され、そのまま公開されることはありません。</p>
        <form @submit.prevent="submitCareer">
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              勤務先名称（必須）（例 xxx株式会社）
            </div>
            <div data-test="company-name-div" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.companyName" v-on:input="setCompanyName" type="text" required minlength="1" maxlength="256" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              部署名（任意）
            </div>
            <div data-test="department-name-div" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.departmentName" v-on:input="setDepartmentName" type="text" minlength="1" maxlength="256" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              勤務地（任意）（例 xxx事業所）
            </div>
            <div data-test="office-div" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.office" v-on:input="setOffice" type="text" minlength="1" maxlength="256" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              入社日（必須）
            </div>
            <div data-test="career-start-year-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="form.careerStartYear" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in yearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div data-test="career-start-year-div" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">年</div>
            <div data-test="career-start-month-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="form.careerStartMonth" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in monthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div data-test="career-start-month-div" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">月</div>
            <div data-test="career-start-day-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="form.careerStartDay" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in dayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div data-test="career-start-day-div" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">日</div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              退社日（任意）
            </div>
            <div data-test="career-end-year-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="form.careerEndYear" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in yearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div data-test="career-end-year-div" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">年</div>
            <div data-test="career-end-month-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="form.careerEndMonth" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in monthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div data-test="career-end-month-div" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">月</div>
            <div data-test="career-end-day-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="form.careerEndDay" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in dayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div data-test="career-end-day-div" class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">日</div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              雇用形態（必須）
            </div>
            <div data-test="contract-type-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-6">
              <select v-model="form.contractType" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option value="regular">正社員</option>
                <option value="contract">契約社員</option>
                <option value="other">その他</option>
              </select>
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              職種（任意）（例 ITエンジニア）
            </div>
            <div data-test="office-div" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.profession" v-on:input="setProfession" type="text" minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              年収（単位：万円）（任意）
            </div>
            <div data-test="annual-incom-in-man-yen-div" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.annualIncomeInManYen" v-on:input="setAnnualIncomeInManYen" type="text" minlength="1" maxlength="5" pattern="\d*" title="半角数字でご入力下さい。" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              管理職区分（必須）
            </div>
            <div data-test="is-manager-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-6">
              <select v-model="form.isManager" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option value="false">非管理職</option>
                <option value="true">管理職</option>
              </select>
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              職位（任意）（例 係長）
            </div>
            <div data-test="position-name-div" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-bind:value="form.positionName" v-on:input="setPositionName" type="text" minlength="1" maxlength="128" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              入社区分（必須）
            </div>
            <div data-test="is-new-graduate-select-div" class="mt-2 w-full text-2xl justify-self-start col-span-6">
              <select v-model="form.isNewGraduate" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option value="true">新卒入社</option>
                <option value="false">中途入社</option>
              </select>
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              備考（任意）
            </div>
            <div data-test="note-div" class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <textarea v-bind:value="form.note" v-on:input="setNote" minlength="1" maxlength="2048" placeholder="例 職場の雰囲気、社風、女性の働きやさ、福利厚生や一日の仕事の流れ等などについて本音でお話できます。" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3"></textarea>
            </div>
            <div data-test="career-image-div" class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              証明書類
            </div>
            <div class="mt-2 text-xl justify-self-start col-span-6 pt-3 pl-3">
              勤務先名称に記載した勤め先にご本人が勤務されていた証明として、書類をアップロードしていただきます。証明書類として、<span class=" text-red-500">名刺、退職証明書または離職票</span>をご利用になれます。証明書類の画像は、<span class=" text-red-500">jpegかつサイズが4MB以下</span>で、勤務先名称に記載した内容とご本人のお名前が記載されている必要があります。離職票をご利用の場合、マイナンバーが記載されていないこと（またはマイナンバーの箇所が隠されていること）を事前にご確認下さい。表面のアップロードは必須、裏面のアップロードは任意となります。
            </div>
            <div data-test="career-image1-div" class="mt-6 pl-3 w-full justify-self-start col-span-1">
              表面
            </div>
            <div class="mt-2 w-full justify-self-start col-span-5 pt-3 rounded bg-gray-200">
              <input type="file" name="image1" v-on:change="onImage1StateChange" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div data-test="career-image2-div" class="mt-6 pl-3 w-full justify-self-start col-span-1">
              裏面
            </div>
            <div class="mt-2 w-full justify-self-start col-span-5 pt-3 rounded bg-gray-200">
              <input type="file" name="image2" v-on:change="onImage2StateChange" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
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
import { usePostCareer } from '@/util/personalized/careers/usePostCareer'
import { useImages } from './useImages'
import { exceedJpegMaxImageSize, isJpegExtension } from '@/util/CheckJpegImage'
import { Message } from '@/util/Message'
import { useCareer } from './useCareer'
import { createDayList } from '@/util/personalized/careers/DayList'
import { createMonthList } from '@/util/personalized/careers/MonthList'
import { createYearList, START_YEAR } from '@/util/personalized/careers/YearList'

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
    const {
      waitingRequestDone,
      postCareerFunc
    } = usePostCareer()
    const {
      images,
      onImage1StateChange,
      onImage2StateChange
    } = useImages()
    const {
      form,
      setCompanyName,
      setDepartmentName,
      setOffice,
      setProfession,
      setAnnualIncomeInManYen,
      setPositionName,
      setNote
    } = useCareer()
    const currentYear = new Date().getFullYear()
    const yearList = createYearList(START_YEAR, currentYear)
    const monthList = createMonthList()
    const dayList = createDayList()
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
      if (images.image1 === null) {
        isHidden.value = false
        errorMessage.value = Message.NO_CAREER_IMAGE1_SELECTED
        return
      }
      if (!isJpegExtension(images.image1.name)) {
        isHidden.value = false
        errorMessage.value = Message.NO_JPEG_EXTENSION_MESSAGE
        return
      }
      if (exceedJpegMaxImageSize(images.image1.size)) {
        isHidden.value = false
        errorMessage.value = Message.EXCEED_MAX_CAREER_IMAGE_SIZE_LIMIT_MESSAGE
        return
      }
      if (images.image2 !== null) {
        if (!isJpegExtension(images.image2.name)) {
          isHidden.value = false
          errorMessage.value = Message.NO_JPEG_EXTENSION_MESSAGE
          return
        }
        if (exceedJpegMaxImageSize(images.image2.size)) {
          isHidden.value = false
          errorMessage.value = Message.EXCEED_MAX_CAREER_IMAGE_SIZE_LIMIT_MESSAGE
          return
        }
      }
      console.log('submitCareer')
      console.log(form.companyName)
      console.log(form.departmentName)
      console.log(form.office)
      console.log(form.contractType)
      console.log(form.profession)
      console.log(form.annualIncomeInManYen)
      console.log(form.isManager)
      console.log(form.positionName)
      console.log(form.isNewGraduate)
      console.log(form.note)
      console.log(form.careerStartYear)
      console.log(form.careerStartMonth)
      console.log(form.careerStartDay)
      console.log(form.careerEndYear)
      console.log(form.careerEndMonth)
      console.log(form.careerEndDay)
    }
    return {
      submitCareer,
      isHidden,
      errorMessage,
      waitingRequestDone,
      yearList,
      monthList,
      dayList,
      form,
      setCompanyName,
      setDepartmentName,
      setOffice,
      setProfession,
      setAnnualIncomeInManYen,
      setPositionName,
      setNote,
      onImage1StateChange,
      onImage2StateChange
    }
  }
})
</script>
