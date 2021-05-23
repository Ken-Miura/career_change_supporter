<template>
  <p v-if="error.exist">{{error.message}}</p>
  <div>
    <form ref="formRef" @submit.prevent="login">
      <input v-model="form.email" type="email" required placeholder="メールアドレス" maxlength=254>
      <!-- TODO: Add password restristion -->
      <input v-model="form.password" type="password" required placeholder="パスワード">
      <button type="submit" :disabled="!form.email || !form.password">ログイン</button>
    </form>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { getSessionState } from '@/store/SessionChecker'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'Login',
  setup () {
    const router = useRouter()
    const store = useStore()
    // TODO: onMounted、onBeforeMount、setupのどれで呼ぶのが正しいか確認する
    onMounted(async () => {
      const sessionState = await getSessionState()
      store.commit('updateSessionState', sessionState)
      if (sessionState === 'active') {
        await router.push('schedule')
      }
    })

    const formRef = ref<HTMLFormElement | null>(null)
    const form = reactive({
      email: '',
      password: ''
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

    const login = async () => {
      if (formRef.value === null) {
        throw new ReferenceError('formRef.value is null')
      }
      if (!formRef.value.checkValidity()) {
        console.log('form.checkValidity: false')
        return
      }
      // Ignore naming convention because "email_address" is JSON param name
      // eslint-disable-next-line
      const data = { email_address: form.email, password: form.password }
      let response
      try {
        response = await fetch('login-request', {
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
      await router.push('schedule')
    }
    return { formRef, form, login, error }
  }
})
</script>
