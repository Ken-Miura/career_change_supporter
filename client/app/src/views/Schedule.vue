<template>
  <p>TODO</p>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { useRouter } from 'vue-router'
import { getSessionState } from '@/store/SessionChecker'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'Schedule',
  setup () {
    const router = useRouter()
    const store = useStore()

    // TODO: onMounted、onBeforeMount、setupのどれで呼ぶのが正しいか確認する
    // onMounted(async () => {
    //   const sessionState = await getSessionState()
    //   store.commit('updateSessionState', sessionState)
    //   if (sessionState !== 'active') {
    //     await router.push('login')
    //   }
    // })
  }
})
</script>
