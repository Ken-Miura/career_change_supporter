<template>
  <div>
    <p v-if="error.exist">{{error.message}}</p>
    <form ref="formRef" @submit.prevent="submitAdviceFee" class="container">
      <h3>1時間あたりの相談料</h3>
      <input v-model="form.adviceFee" type="text" inputmode="numeric" pattern="\d*" required placeholder="相談料">
      <button type="submit" :disabled="!form.adviceFee">相談料の更新</button>
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
      adviceFee: ''
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
    const submitAdviceFee = async () => {
      if (formRef.value === null) {
        throw new ReferenceError('formRef.value is null')
      }
      if (!formRef.value.checkValidity()) {
        console.log('form.checkValidity: false')
        return
      }
      // Ignore naming convention because "email_address" is JSON param name
      // eslint-disable-next-line
      const data = { advice_fee: Number(form.adviceFee) }
      let response
      try {
        response = await fetch('advice-fee', {
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
    return { formRef, form, submitAdviceFee, error }
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
