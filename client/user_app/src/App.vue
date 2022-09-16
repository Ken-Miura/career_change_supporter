<template>
  <router-view/>
</template>

<script lang="ts">
import { defineComponent, onMounted } from 'vue'
import { useStore } from 'vuex'
import { SET_PAY_JP } from './store/mutationTypes'

export default defineComponent({
  name: 'App',
  setup () {
    const store = useStore()
    onMounted(() => {
      if (store.state.payJp !== null) {
        return
      }
      const payJpPubKey = process.env.VUE_APP_PAYJP_PUBLIC_KEY
      // index.htmlのhead内でasync/deferなしのpay.jp js v2のscriptタグを読み込んでいるため、
      // window.Payjpは定義が存在することを前提にコードを書いても問題ない
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const payjp = window.Payjp(payJpPubKey)
      store.commit(SET_PAY_JP, payjp)
    })
  }
})
</script>
