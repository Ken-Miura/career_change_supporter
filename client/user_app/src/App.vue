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
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const payjp = window.Payjp('TODO: place public_key')
      store.commit(SET_PAY_JP, payjp)
    })
  }
})
</script>
