<template>
  <div>
    <h1>This is a search page</h1>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getSessionState } from '@/store/SessionChecker'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'Search',
  setup () {
    const router = useRouter()
    const store = useStore()

    onMounted(async () => {
      const sessionState = await getSessionState()
      store.commit('updateSessionState', sessionState)
      if (sessionState !== 'active') {
        await router.push('login')
      }
    })
  }
})
</script>
