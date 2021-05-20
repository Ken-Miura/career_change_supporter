<template>
  <p v-if="registration.run">{{registration.message}}</p>
  <div>
    <form ref="formRef" @submit.prevent="register">
      <input v-model="form.email" type="email" required placeholder="メールアドレス" maxlength=254>
      <button type="submit" :disabled="!form.email">アカウント作成</button>
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
  methods: {
    async register () {
      const form = this.form
      const registration = this.registration
      // Ignore naming convention because "email_address" is JSON param name
      // eslint-disable-next-line
      const data = { email_address: form.email }
      registration.run = true
      let response
      try {
        response = await fetch('registration-request', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json; charset=utf-8' },
          body: JSON.stringify(data)
        })
      } catch (e) {
        console.log(`failed to get response: ${e}`)
        registration.message = '通信エラーが発生しました。インターネットに接続できているか確認してください。'
        return
      }
      registration.message = await this.createMessage(response)
    },
    async createMessage (response: Response): Promise<string> {
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
  },
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
      email: ''
    })
    const registration = reactive({
      run: false,
      message: ''
    })
    return { formRef, form, registration }
  }
})
</script>
