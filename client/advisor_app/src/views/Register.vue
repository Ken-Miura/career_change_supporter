<template>
  <p v-if="registration.run">{{registration.message}}</p>
  <div>
    <form ref="formRef" @submit.prevent="register">
      <input v-model="form.email" type="email" required placeholder="メールアドレス" maxlength=254>
      <!-- TODO: Add password restristion -->
      <input v-model="form.password" type="password" required placeholder="パスワード">
      <button type="submit" :disabled="!form.email || !form.password">アカウント作成</button>
    </form>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { getSessionState } from '@/store/SessionChecker'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'Register',
  setup () {
    const router = useRouter()
    const store = useStore()

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
    const registration = reactive({
      run: false,
      message: ''
    })

    const createMessage = async (response: Response): Promise<string> => {
      if (response.ok) {
        const result = await response.json()
        return result.message
      } else {
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
    }

    const register = async () => {
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
      registration.run = true
      let response
      try {
        response = await fetch('temporary-account-creation', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json; charset=utf-8' },
          body: JSON.stringify(data)
        })
      } catch (e) {
        console.log(`failed to get response: ${e}`)
        registration.message = '通信エラーが発生しました。インターネットに接続できているか確認してください。'
        return
      }
      registration.message = await createMessage(response)
    }
    return { formRef, form, register, registration }
  }
})
</script>
