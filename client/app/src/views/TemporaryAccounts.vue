<template>
  <div>
    <p>{{ result.message }}</p>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { LocationQuery, useRouter } from 'vue-router'
import { getSessionState } from '@/store/SessionChecker'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'TemporaryAccounts',
  setup () {
    // const result = reactive({
    //   message: ''
    // })
    // const createErrorMessage = async (response: Response): Promise<string> => {
    //   try {
    //     const err = await response.json()
    //     if (err.code !== undefined && err.message !== undefined) {
    //       return `${err.message} (エラーコード: ${err.code})`
    //     } else {
    //       return `予期せぬエラーが発生しました。${err.message} (エラーコード: ${err.code})`
    //     }
    //   } catch (e) {
    //     return `予期せぬエラーが発生しました。${e}`
    //   }
    // }
    // const createAccount = async (query: LocationQuery) => {
    //   let response
    //   try {
    //     response = await fetch('account-creation', {
    //       method: 'POST',
    //       headers: { 'Content-Type': 'application/json; charset=utf-8' },
    //       body: JSON.stringify(query)
    //     })
    //   } catch (e) {
    //     console.log(`failed to get response: ${e}`)
    //     result.message = '通信エラーが発生しました。インターネットに接続できているか確認してください。'
    //     return
    //   }
    //   if (response.ok) {
    //     const resJson = await response.json()
    //     result.message = resJson.message
    //   } else {
    //     result.message = await createErrorMessage(response)
    //   }
    // }
    // const router = useRouter()
    // const store = useStore()
    // // TODO: onMounted、onBeforeMount、setupのどれで呼ぶのが正しいか確認する
    // onMounted(async () => {
    //   const sessionState = await getSessionState()
    //   store.commit('updateSessionState', sessionState)
    //   if (sessionState === 'active') {
    //     await router.push('schedule')
    //     return
    //   }
    //   const query = router.currentRoute.value.query
    //   await createAccount(query)
    // })
    // return { result }
  }
})
</script>
