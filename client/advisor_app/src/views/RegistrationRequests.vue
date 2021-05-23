<template>
  <p v-if="error.exist">{{error.message}}</p>
  <div v-if="!error.exist">
    <p>{{result.emailAddress}}</p>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, reactive } from 'vue'
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
    return { error, result }
  }
})
</script>
