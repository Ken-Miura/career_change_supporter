<template>
  <p v-if="error.exist">{{error.message}}</p>
  <div v-if="!error.exist">
      <form ref="formRef" class="container" @submit.prevent="submitData">
        <p id="description">下記の必要な情報を入力し、登録を完了させてください。</p>

        <div id="accountInfoContainer">
          <p id="emailAddressLabel">メールアドレス</p>
          <p id="emailAddress">{{form.emailAddress}}</p>
          <p id="passwordLabel">パスワード</p>
          <!-- TODO: Add password restristion -->
          <input id="password" v-model="form.password" type="password" required>
        </div>

        <div id="advisorInfoContainer">
          <div id="nameContainer">
            <p id="nameTitle">お名前 (漢字)</p>
            <p id="lastName">姓: <input v-model="form.lastName" type = "text" required></p>
            <p id="firstName">名: <input v-model="form.firstName" type = "text" required></p>
          </div>
          <div id="nameFuriganaContainer">
            <p id="nameFuriganaTitle">お名前 (フリガナ)</p>
            <p id="lastNameFurigana">セイ: <input v-model="form.lastNameFurigana" type = "text" required></p>
            <p id="firstNameFurigana">メイ: <input v-model="form.firstNameFurigana" type = "text" required></p>
          </div>
          <div id="telephoneNumberContainer">
            <p id="telephoneNumberTitle">電話番号</p>
            <input id="telephoneNumber" v-model="form.telephonNumber" type = "text" required>
          </div>
          <div id="dateOfBirthContainer">
            <p id="dateOfBirthTitle">生年月日</p>
            <p id="dateOfBirth"><select v-model="form.year">
              <option v-for="year in yearList" :key="year" :value="year">{{ year }}</option>
            </select> 年
            <select id="month" v-model="form.month">
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
            <select id="day" v-model="form.day">
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
          <div id="addressContainer">
            <p id="addressTitle">住所</p>
            <p id="prefecture">都道府県:</p><input id="prefectureInput" v-model="form.prefecture" type = "text" required>
            <p id="city">市区町村:</p><input id="cityInput" v-model="form.city" type = "text" required>
            <p id="addressLine1">それ以降の住所:</p><input id="addressLine1Input" v-model="form.addressLine1" type = "text" required>
            <p id="addressLine2">建物名・号室:</p><input id="addressLine2Input" v-model="form.addressLine2" type = "text">
          </div>
          <div id="identificationContainer">
            <p id="identificationTitle">身分証明書</p>
            <!-- TODO: レイアウトを考える -->
            <!--<p id="identificationDescription">運転免許証、保険証、在留カードは表面と裏面、<br>パスポートは顔写真記載面と現住所記載面をアップロードしてください<br>（保険証は、保険証番号をマスキングした状態でアップロードください）</p>  -->
            <p id="image1">画像1: <input type="file" @change="onImage1StateChange" name="file1"/></p>
            <p id="image2">画像2: <input type="file" @change="onImage2StateChange" name="file2"/></p>
          </div>
        </div>
        <button type="submit" :disabled="!form.password">登録</button>
      </form>
      <p v-if="registration.run">{{registration.message}}</p>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, reactive, ref } from 'vue'
import { LocationQuery, useRouter } from 'vue-router'
import { getSessionState } from '@/store/SessionChecker'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'RegistrationRequests',
  setup () {
    const error = reactive({
      exist: false,
      message: ''
    })
    const registration = reactive({
      run: false,
      message: ''
    })
    const formRef = ref<HTMLFormElement | null>(null)
    const form = reactive({
      emailAddress: '',
      password: '',
      lastName: '',
      firstName: '',
      lastNameFurigana: '',
      firstNameFurigana: '',
      telephonNumber: '',
      year: '',
      month: '',
      day: '',
      prefecture: '',
      city: '',
      addressLine1: '',
      addressLine2: '',
      image1: null as FileList | null,
      image2: null as FileList | null
    })
    const yearList = reactive([] as number[])
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

    const checkIfRequestIdExpires = async (query: LocationQuery) => {
      let response
      try {
        response = await fetch('registration-request-id-check', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json; charset=utf-8' },
          body: JSON.stringify(query)
        })
      } catch (e) {
        console.log(`failed to get response: ${e}`)
        error.exist = true
        error.message = '通信エラーが発生しました。インターネットに接続できているか確認してください。'
        return
      }
      if (response.ok) {
        error.exist = false
        error.message = ''
        const resJson = await response.json()
        form.emailAddress = resJson.email_address
      } else {
        error.exist = true
        error.message = await createErrorMessage(response)
      }
    }

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

    const router = useRouter()
    const store = useStore()
    // TODO: onMounted、onBeforeMount、setupのどれで呼ぶのが正しいか確認する
    onMounted(async () => {
      const sessionState = await getSessionState()
      store.commit('updateSessionState', sessionState)
      if (sessionState === 'active') {
        await router.push('schedule')
        return
      }
      const start = 1950
      const date = new Date()
      const end = date.getFullYear() - 18
      for (let i = start; i < (end + 1); i++) {
        yearList.push(i)
      }
      await checkIfRequestIdExpires(router.currentRoute.value.query)
    })

    const submitData = async () => {
      const query = router.currentRoute.value.query
      const data = {
        id: query.id,
        password: form.password,
        last_name: form.lastName, // eslint-disable-line
        first_name: form.firstName, // eslint-disable-line
        last_name_furigana: form.lastNameFurigana, // eslint-disable-line
        first_name_furigana: form.firstNameFurigana, // eslint-disable-line
        telephone_number: form.telephonNumber, // eslint-disable-line
        year_of_birth: parseInt(form.year), // eslint-disable-line
        month_of_birth: parseInt(form.month), // eslint-disable-line
        day_of_birth: parseInt(form.day), // eslint-disable-line
        prefecture: form.prefecture, // eslint-disable-line
        city: form.city, // eslint-disable-line
        address_line1: form.addressLine1, // eslint-disable-line
        address_line2: form.addressLine2, // eslint-disable-line
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
      registration.run = true
      let response
      try {
        response = await fetch('account-creation-request', {
          method: 'POST',
          body: formData
        })
      } catch (e) {
        console.log(`failed to get response: ${e}`)
        registration.message = '通信エラーが発生しました。インターネットに接続できているか確認してください。'
        return
      }
      if (response.ok) {
        const res = await response.json()
        registration.message = await res.message
      } else {
        registration.message = await createErrorMessage(response)
      }
    }
    return { error, formRef, form, yearList, onImage1StateChange, onImage2StateChange, submitData, registration }
  }
})
</script>

<style scoped>
#accountInfoContainer {
  display: grid;
  grid-template-columns: mim-content;
  align-items: center;
  justify-items: start;
}

#advisorInfoContainer {
  display: grid;
  grid-template-columns: 1fr;
  align-items: center;
  justify-items: center;
  row-gap: 0.5ex;
}
#nameContainer {
  display: grid;
  grid-template-columns: 1fr;
  align-items: center;
  justify-items: start;
}
#nameFuriganaContainer {
  display: grid;
  grid-template-columns: 1fr;
  align-items: center;
  justify-items: start;
}
#telephoneNumberContainer {
  display: grid;
  grid-template-columns: 1fr;
  align-items: center;
  justify-items: start;
}
#dateOfBirthContainer {
  display: grid;
  grid-template-columns: 1fr;
  align-items: center;
  justify-items: start;
}
#addressContainer {
  display: grid;
  grid-template-columns: max-content 1fr;
  align-items: center;
  justify-items: start;
  column-gap: 0.5ex;
}
#addressTitle {
  grid-row: 1;
  grid-column: 1/3;
}
#prefecture {
  grid-row: 2;
  grid-column: 1;
}
#prefectureInput {
  grid-row: 2;
  grid-column: 2;
}
#city {
  grid-row: 3;
  grid-column: 1;
}
#cityInput {
  grid-row: 3;
  grid-column: 2;
}
#addressLine1 {
  grid-row: 4;
  grid-column: 1;
}
#addressLine1Input {
  grid-row: 4;
  grid-column: 2;
}
#addressLine2 {
  grid-row: 5;
  grid-column: 1;
}
#addressLine2Input {
  grid-row: 5;
  grid-column: 2;
}
#identificationContainer {
  display: grid;
  grid-template-columns: 1fr;
  align-items: center;
  justify-items: start;
}

.container {
  display: grid;
  grid-template-columns: 1fr;
  align-items: center;
  justify-items: center;
  row-gap: 2ex;
}
#description {
    grid-row: 1;
    grid-column: 1/2;
}
#accountInfoContainer {
    grid-row: 2;
}
#advisorInfoContainer {
    grid-row: 3;
}
</style>
