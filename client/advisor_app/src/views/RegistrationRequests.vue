<template>
  <p v-if="error.exist">{{error.message}}</p>
  <div v-if="!error.exist">
    <p>メールアドレス: {{result.emailAddress}}</p>
    <form ref="formRef" @submit.prevent="submitRegistrationInformation">
      <!-- TODO: Add password restristion -->
      <input v-model="form.password" type="password" required placeholder="パスワード">
      <p>アドバイザー情報</p>
      <input v-model="form.lastName" type = "text" required placeholder="姓">
      <input v-model="form.firstName" type = "text" required placeholder="名">
      <!-- TODO: 口座名義必須ならいらない？ -->
      <input v-model="form.lastNameReading" type = "text" required placeholder="セイ">
      <input v-model="form.firstNameReading" type = "text" required placeholder="メイ">
      <input v-model="form.telephonNumber" type = "text" required placeholder="電話番号">
      <input v-model="form.address" type = "text" required placeholder="住所">
      <input v-model="form.dateOfBirth" type = "text" required placeholder="生年月日">
      <p>身分証明書</p>
      <div>
        <p>表面: </p><input type="file" @change="test1" name="file1"/>
      </div>
      <div>
        <p>裏面: </p><input type="file" @change="test2" name="file2"/>
      </div>
      <p>振込口座情報</p>
      <input v-model="form.bankCode" type = "text" pattern="[0-9]{4}" required placeholder="銀行コード">
      <input v-model="form.bankBranchCode" type = "text" pattern="[0-9]{3}" required placeholder="支店コード">
      <!-- 参考情報：メルカリの振込申請での預金種別は、普通預金、当座預金、貯蓄預金のみ。当座を登録するようなユーザを想定していない。普通預金だけでいいかなあ -->
      <!--<input v-model="form.bankAccountType" type = "text" required placeholder="預金種別">-->
      <p>預金種別: 普通</p>
      <!-- 参考情報：普通預金で5桁、6桁や当座預金で3桁、4桁があるが0で先頭を埋めれば良い。ゆうちょ銀行の普通預金の8桁の場合、最後の1桁の1を削除すればよい -->
      <input v-model="form.bankAccountNumber" type = "text" pattern="[0-9]{7}" required placeholder="口座番号">
      <input v-model="form.bankAccountHolderLastName" type = "text" required placeholder="口座名義（セイ）">
      <input v-model="form.bankAccountHolderFirstName" type = "text" required placeholder="口座名義（メイ）">
      <!-- 料金設定の項目 -->
      <input v-model="form.fee" type = "text" required placeholder="料金設定">
      <button type="submit" :disabled="!form.password">登録</button>
    </form>
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
    const result = reactive({
      emailAddress: ''
    })
    const formRef = ref<HTMLFormElement | null>(null)
    const form = reactive({
      password: '',
      lastName: '',
      firstName: '',
      lastNameReading: '',
      firstNameReading: '',
      address: '',
      dateOfBirth: '',
      bankCode: '',
      bankBranchCode: '',
      bankAccountNumber: '',
      bankAccountHolderLastName: '',
      bankAccountHolderFirstName: '',
      fee: '',
      identificationHeads: null as FileList | null,
      identificationTails: null as FileList | null
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
        result.emailAddress = resJson.email_address
      } else {
        error.exist = true
        error.message = await createErrorMessage(response)
      }
    }

    const test1 = (event: Event) => {
      console.log('test1')
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
      form.identificationHeads = files
    }

    const test2 = (event: Event) => {
      console.log('test2')
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
      form.identificationHeads = files
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
      await checkIfRequestIdExpires(router.currentRoute.value.query)
    })
    return { error, result, formRef, form, test1, test2 }
  }
})
</script>
