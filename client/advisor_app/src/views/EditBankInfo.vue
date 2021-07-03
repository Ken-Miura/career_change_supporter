<template>
  <div>
    <p v-if="error.exist">{{error.message}}</p>
    <form ref="formRef" @submit.prevent="submitBankInfo" class="container">
      <h3>口座情報</h3>
      <input v-model="form.bankCode" type = "text" required placeholder="銀行コード">
      <input v-model="form.bankBranchCode" type = "text" required placeholder="支店コード">
      <p>口座種別：普通（口座種別は普通のみサポートしています）</p>
      <input v-model="form.bankAccountNumber" type = "text" required placeholder="口座番号">
      <p>振込先名義は、登録されている身分情報と同じ方に限定されます。</p>
      <input v-model="form.bankAccountHolderName" type = "text" required placeholder="口座名義人">
      <button type="submit" :disabled="!(form.bankCode && form.bankBranchCode && form.bankAccountNumber && form.bankAccountHolderName)">口座情報更新</button>
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
      bankCode: '',
      bankBranchCode: '',
      bankAccountNumber: '',
      bankAccountHolderName: ''
    })

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
    })
    const submitBankInfo = async () => {
      if (formRef.value === null) {
        throw new ReferenceError('formRef.value is null')
      }
      if (!formRef.value.checkValidity()) {
        console.log('form.checkValidity: false')
        return
      }
      // Ignore naming convention because "email_address" is JSON param name
      // eslint-disable-next-line
      const data = { bank_code: form.bankCode, bank_branch_code: form.bankBranchCode, bank_account_number: form.bankAccountNumber, bank_account_holder_name: form.bankAccountHolderName }
      let response
      try {
        response = await fetch('bank-info', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json; charset=utf-8' },
          body: JSON.stringify(data)
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
    return { formRef, form, submitBankInfo, error }
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
</style>
