<template>
  <div>
    <p v-if="error.exist">{{error.message}}</p>
    <form ref="formRef" @submit.prevent="submitCareerInfo" class="container">
      <h3>経歴情報</h3>
      <input v-model="form.companyName" type = "text" required placeholder="会社名">
      <input v-model="form.departmentName" type = "text" required placeholder="部署名">
      <input v-model="form.office" type = "text" required placeholder="勤務先事業所">
      <p>雇用種別</p>
      <select v-model="form.contractType">
        <option value="正社員">正社員</option>
        <option value="契約社員">契約社員</option>
        <option value="その他">その他</option>
      </select>
      <div id="startDateContainer">
        <p>入社日</p>
        <p>
          <select v-model="form.startYear"><option v-for="year in yearList" :key="year" :value="year">{{ year }}</option></select> 年
          <select id="startMonth" v-model="form.startMonth">
            <option value="1">1</option>
            <option value="2">2</option>
            <option value="3">3</option>
            <option value="4">4</option>
            <option value="5">5</option>
            <option value="6">6</option>
            <option value="7">7</option>
            <option value="8">8</option>
            <option value="9">9</option>
            <option value="10">10</option>
            <option value="11">11</option>
            <option value="12">12</option>
          </select> 月
          <select id="startDay" v-model="form.startDay">
            <option value="1">1</option>
            <option value="2">2</option>
            <option value="3">3</option>
            <option value="4">4</option>
            <option value="5">5</option>
            <option value="6">6</option>
            <option value="7">7</option>
            <option value="8">8</option>
            <option value="9">9</option>
            <option value="10">10</option>
            <option value="11">11</option>
            <option value="12">12</option>
            <option value="13">13</option>
            <option value="14">14</option>
            <option value="15">15</option>
            <option value="16">16</option>
            <option value="17">17</option>
            <option value="18">18</option>
            <option value="19">19</option>
            <option value="20">20</option>
            <option value="21">21</option>
            <option value="22">22</option>
            <option value="23">23</option>
            <option value="24">24</option>
            <option value="25">25</option>
            <option value="26">26</option>
            <option value="27">27</option>
            <option value="28">28</option>
            <option value="29">29</option>
            <option value="30">30</option>
            <option value="31">31</option>
          </select> 日</p>
      </div>
      <div id="endDateContainer">
        <p>退社日</p>
        <p>
          <select v-model="form.endYear"><option v-for="year in yearList" :key="year" :value="year">{{ year }}</option></select> 年
          <select id="startMonth" v-model="form.endMonth">
            <option value="1">1</option>
            <option value="2">2</option>
            <option value="3">3</option>
            <option value="4">4</option>
            <option value="5">5</option>
            <option value="6">6</option>
            <option value="7">7</option>
            <option value="8">8</option>
            <option value="9">9</option>
            <option value="10">10</option>
            <option value="11">11</option>
            <option value="12">12</option>
          </select> 月
          <select id="startDay" v-model="form.endDay">
            <option value="1">1</option>
            <option value="2">2</option>
            <option value="3">3</option>
            <option value="4">4</option>
            <option value="5">5</option>
            <option value="6">6</option>
            <option value="7">7</option>
            <option value="8">8</option>
            <option value="9">9</option>
            <option value="10">10</option>
            <option value="11">11</option>
            <option value="12">12</option>
            <option value="13">13</option>
            <option value="14">14</option>
            <option value="15">15</option>
            <option value="16">16</option>
            <option value="17">17</option>
            <option value="18">18</option>
            <option value="19">19</option>
            <option value="20">20</option>
            <option value="21">21</option>
            <option value="22">22</option>
            <option value="23">23</option>
            <option value="24">24</option>
            <option value="25">25</option>
            <option value="26">26</option>
            <option value="27">27</option>
            <option value="28">28</option>
            <option value="29">29</option>
            <option value="30">30</option>
            <option value="31">31</option>
          </select> 日</p>
      </div>
      <input v-model="form.profession" type = "text" required placeholder="職種">
      <input v-model="form.positionName" type = "text" required placeholder="職位名">
      <select v-model="form.isManager">
        <option value="false">非管理職</option>
        <option value="true">管理職</option>
      </select>
      <input v-model="form.annualIncomeInManYen" type = "text" inputmode="numeric" pattern="\d*" required placeholder="年収">
      <select v-model="form.isNewGraduate">
        <option value="false">新卒入社</option>
        <option value="true">中途入社</option>
      </select>
      <input v-model="form.note" type = "text" required placeholder="備考">
      <p id="image1">画像1: <input type="file" @change="onImage1StateChange" name="file1"/></p>
      <p id="image2">画像2: <input type="file" @change="onImage2StateChange" name="file2"/></p>
      <button type="submit">経歴情報作成</button>
    </form>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getSessionState } from '@/store/SessionChecker'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'Profile',
  setup () {
    const formRef = ref<HTMLFormElement | null>(null)
    const form = reactive({
      companyName: '',
      departmentName: '',
      office: '',
      contractType: '',
      startYear: '',
      startMonth: '',
      startDay: '',
      endYear: '',
      endMonth: '',
      endDay: '',
      profession: '',
      positionName: '',
      isManager: '',
      annualIncomeInManYen: '',
      isNewGraduate: '',
      note: '',
      image1: null as FileList | null,
      image2: null as FileList | null
    })
    const yearList = reactive([] as number[])

    const error = reactive({
      exist: false,
      message: ''
    })

    const createErrorMessage = async (response: Response): Promise<string> => {
      try {
        const err = await response.json()
        if (err.code !== undefined && err.message !== undefined) {
          return `${err.message} (エラーコード: ${err.code})`
        } else {
          return `予期せぬエラーが発生しました。${err.message} (エラーコード: ${err.code})`
        }
      } catch (e) {
        return `予期せぬエラーが発生しました。${e}`
      }
    }

    const router = useRouter()
    const store = useStore()
    // TODO: onMounted、onBeforeMount、setupのどれで呼ぶのが正しいか確認する
    onMounted(async () => {
      const sessionState = await getSessionState()
      store.commit('updateSessionState', sessionState)
      if (sessionState !== 'active') {
        await router.push('login')
      }
      const date = new Date()
      const end = date.getFullYear()
      const start = end - 52
      for (let i = start; i < (end + 1); i++) {
        yearList.push(i)
      }
    })

    const onImage1StateChange = (event: Event) => {
      console.log('onImage1StateChange')
      if (event === null) {
        console.log('event === null')
        return
      }
      if (event.target === null) {
        console.log('event.target === null')
        return
      }
      const files = (event.target as HTMLInputElement).files
      if (files === null) {
        console.log('files === null')
        return
      }
      form.image1 = files
    }

    const onImage2StateChange = (event: Event) => {
      console.log('onImage2StateChange')
      if (event === null) {
        console.log('event === null')
        return
      }
      if (event.target === null) {
        console.log('event.target === null')
        return
      }
      const files = (event.target as HTMLInputElement).files
      if (files === null) {
        console.log('files === null')
        return
      }
      form.image2 = files
    }

    const submitCareerInfo = async () => {
      if (formRef.value === null) {
        throw new ReferenceError('formRef.value is null')
      }
      if (!formRef.value.checkValidity()) {
        console.log('form.checkValidity: false')
        return
      }
      // Ignore naming convention because "email_address" is JSON param name
      // eslint-disable-next-line
      const data = { 
        company_name: form.companyName, // eslint-disable-line
        department_name: form.departmentName, // eslint-disable-line
        office: form.office, // eslint-disable-line
        contract_type: form.contractType, // eslint-disable-line
        start_year: parseInt(form.startYear), // eslint-disable-line
        start_month: parseInt(form.startMonth), // eslint-disable-line
        start_day: parseInt(form.startDay), // eslint-disable-line
        end_year: parseInt(form.endYear), // eslint-disable-line
        end_month: parseInt(form.endMonth), // eslint-disable-line
        end_day: parseInt(form.endDay), // eslint-disable-line
        profession: form.profession, // eslint-disable-line
        position_name: form.positionName, // eslint-disable-line
        annual_income_in_man_yen: parseInt(form.annualIncomeInManYen), // eslint-disable-line
        note: form.note, // eslint-disable-line
        is_manager: (form.isManager === 'true'), // eslint-disable-line
        is_new_graduate: (form.isNewGraduate === 'true') // eslint-disable-line
      }

      const formData = new FormData()
      formData.append('parameter', JSON.stringify(data))
      const files1 = form.image1
      if (files1 !== null) {
        const file = files1[0]
        formData.append('image1', file)
      }
      const files2 = form.image2
      if (files2 !== null) {
        const file = files2[0]
        formData.append('image2', file)
      }
      let response
      try {
        response = await fetch('career-registeration', {
          method: 'POST',
          body: formData
        })
      } catch (e) {
        console.log(`failed to get response: ${e}`)
        error.exist = true
        error.message = '通信エラーが発生しました。インターネットに接続できているか確認してください。'
        return
      }
      if (!response.ok) {
        error.exist = true
        error.message = await createErrorMessage(response)
        return
      }
      // NOTE: Credential information will be stored in cookie as session id
      await router.push('profile')
    }
    return { formRef, form, submitCareerInfo, onImage1StateChange, onImage2StateChange, yearList, error }
  }
})
</script>

<style scoped>
.container {
  display: flex;
  justify-content: center;
  align-items: center;
  flex-direction: column;
}
#startDateContainer {
  display: grid;
  grid-template-columns: 1fr;
  align-items: center;
  justify-items: start;
}
#endDateContainer {
  display: grid;
  grid-template-columns: 1fr;
  align-items: center;
  justify-items: start;
}
</style>
