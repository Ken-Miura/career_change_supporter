<template>
  <div>
    <form @submit.prevent="submitBankInfo" class="container">
      <h3>口座情報</h3>
      <input v-model="form.bankCode" type = "text" required placeholder="銀行コード">
      <input v-model="form.bankBranchCode" type = "text" required placeholder="支店コード">
      <input v-model="form.bankAccountType" type = "text" required placeholder="口座種別">
      <input v-model="form.bankAccountNumber" type = "text" required placeholder="口座番号">
      <p>振込先名義は、登録されている身分情報と同じ方に限定されます</p>
      <p>{{profile.furigana}}</p>
      <button type="submit">口座情報更新</button>
    </form>
  </div>
</template>

<script lang="ts">
import { defineComponent, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getSessionState } from '@/store/SessionChecker'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'Profile',
  setup () {
    const profile = reactive({
      furigana: ''
    })
    const form = reactive({
      bankCode: '',
      bankBranchCode: '',
      bankAccountType: '',
      bankAccountNumber: ''
    })

    const router = useRouter()
    const store = useStore()
    // TODO: onMounted、onBeforeMount、setupのどれで呼ぶのが正しいか確認する
    onMounted(async () => {
      const sessionState = await getSessionState()
      store.commit('updateSessionState', sessionState)
      if (sessionState !== 'active') {
        await router.push('login')
        return
      }
      const response = await fetch('profile-information', {
        method: 'GET'
      })
      if (!response.ok) {
        // TODO: エラーハンドリング
        return
      }
      const userInfo = await response.json()
      profile.furigana = userInfo.last_name_furigana + '　' + userInfo.first_name_furigana
    })
    const submitBankInfo = async () => {
      alert('submitBankInfo')
    }
    return { profile, form, submitBankInfo }
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
