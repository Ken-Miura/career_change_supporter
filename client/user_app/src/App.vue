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
      // index.htmlのhead内でasync/deferなしのpay.jp js v2のscriptタグを読み込んでいるため、
      // window.Payjpは定義が存在することを前提にコードを書いても問題ない
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const payjp = window.Payjp('TODO: place public_key')
      store.commit(SET_PAY_JP, payjp)
    })
  }
})
</script>
