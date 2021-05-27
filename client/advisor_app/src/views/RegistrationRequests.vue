<template>
  <p v-if="error.exist">{{error.message}}</p>
  <div v-if="!error.exist">
      <form ref="formRef" class="container" @submit.prevent="submitRegistrationInformation">
        <p id="description">下記の必要な情報を入力し、登録を完了させてください。</p>

        <p id="emailAddressLabel">メールアドレス:</p>
        <p id="emailAddress">{{form.emailAddress}}</p>

        <p id="passwordLabel">パスワード:</p>
        <!-- TODO: Add password restristion -->
        <input id="password" v-model="form.password" type="password" required>

        <p id="lastNameLabel">姓:</p>
        <input id="lastName" v-model="form.lastName" type = "text" required>

        <!-- TODO: 口座名義必須ならいらない？ -->
        <p id="lastNameReadingLabel">セイ:</p>
        <input id="lastNameReading" v-model="form.lastNameReading" type = "text" required>

        <p id="firstNameLabel">名:</p>
        <input id="firstName" v-model="form.firstName" type = "text" required>

        <!-- TODO: 口座名義必須ならいらない？ -->
        <p id="firstNameReadingLabel">メイ:</p>
        <input id="firstNameReading" v-model="form.lastNameReading" type = "text" required>

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
        <input v-model="form.bankCode" type = "text" pattern="[0-9]{4}" title="4桁の数字を入力してください。" required placeholder="銀行コード">
        <input v-model="form.bankBranchCode" type = "text" pattern="[0-9]{3}" title="3桁の数字を入力してください。" required placeholder="支店コード">
        <!-- 参考情報：メルカリの振込申請での預金種別は、普通預金、当座預金、貯蓄預金のみ。当座を登録するようなユーザを想定していない。普通預金だけでいいかなあ -->
        <!--<input v-model="form.bankAccountType" type = "text" required placeholder="預金種別">-->
        <p>預金種別: 普通</p>
        <!-- 参考情報：普通預金で5桁、6桁や当座預金で3桁、4桁があるが0で先頭を埋めれば良い。ゆうちょ銀行の普通預金の8桁の場合、最後の1桁の1を削除すればよい -->
        <input v-model="form.bankAccountNumber" type = "text" pattern="[0-9]{7}" title="7桁の数字を入力してください。" required placeholder="口座番号">
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
    const formRef = ref<HTMLFormElement | null>(null)
    const form = reactive({
      emailAddress: '',
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
        form.emailAddress = resJson.email_address
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
    return { error, formRef, form, test1, test2 }
  }
})
</script>

<style scoped>
.container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  align-items: center;
  column-gap: 0.5ex;

}
#description {
    justify-self: center;
    grid-row: 1;
    grid-column: 1/3;
}
#emailAddressLabel {
    align-self: baseline;
    justify-self: end;
    grid-row: 2;
    grid-column: 1;
}
#emailAddress {
    align-self: baseline;
    justify-self: start;
    grid-row: 2;
    grid-column: 2;
}
#passwordLabel {
    justify-self: end;
    grid-row: 3;
    grid-column: 1;
}
#password {
    justify-self: start;
    grid-row: 3;
    grid-column: 2;
}
#lastNameLabel {
    justify-self: end;
    grid-row: 4;
    grid-column: 1;
}
#lastName {
    justify-self: start;
    grid-row: 4;
    grid-column: 2;
}
#lastNameReadingLabel {
    justify-self: end;
    grid-row: 5;
    grid-column: 1;
}
#lastNameReading {
    justify-self: start;
    grid-row: 5;
    grid-column: 2;
}
#firstNameLabel {
    justify-self: end;
    grid-row: 6;
    grid-column: 1;
}
#firstName {
    justify-self: start;
    grid-row: 6;
    grid-column: 2;
}
#firstNameReadingLabel {
    justify-self: end;
    grid-row: 7;
    grid-column: 1;
}
#firstNameReading {
    justify-self: start;
    grid-row: 7;
    grid-column: 2;
}
</style>
