<template>
  <router-view/>
</template>

<script lang="ts">
import { defineComponent, onMounted } from 'vue'
import { useStore } from 'vuex'
import { SET_PAY_JP } from './store/mutationTypes'
import { loadScript } from 'vue-plugin-load-script'

export default defineComponent({
  name: 'App',
  setup () {
    const store = useStore()
    onMounted(async () => {
      if (store.state.payJp !== null) {
        return
      }
      const payJpJsUrl = 'https://js.pay.jp/v2/pay.js'
      try {
        await loadScript(payJpJsUrl)
        const payJpPubKey = process.env.VUE_APP_PAYJP_PUBLIC_KEY
        // https://js.pay.jp/v2/pay.js 内でPayjpオブジェクトをwindowに追加している。
        // コンパイラは、ホストされたJavascriptファイル内で行われている処理を検知できないため、明示的にチェックを無視する。
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const payjp = window.Payjp(payJpPubKey)
        store.commit(SET_PAY_JP, payjp)
      } catch (e) {
        console.error(`failed to load Javascript file from ${payJpJsUrl}`)
      }
    })
  }
})
</script>
