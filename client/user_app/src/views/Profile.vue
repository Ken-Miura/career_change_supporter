<template>
  <div>
    <h1>Profile</h1>
    <p>{{profile.id}}</p>
    <p>{{profile.email}}</p>
  </div>
</template>

<script lang="ts">
import { defineComponent, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getSessionState } from '@/store/SessionChecker'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'Profile',
  setup () {
    const profile = reactive({
      id: '',
      email: ''
    })

    const router = useRouter()
    const store = useStore()

    onMounted(async () => {
      const sessionState = await getSessionState()
      store.commit('updateSessionState', sessionState)
      if (sessionState !== 'active') {
        await router.push('login')
        return
      }
      const response = await fetch('profile-information', {
        method: 'GET'
      })
      if (!response.ok) {
        profile.id = 'error: failed to get id'
        profile.email = 'error: failed to get email'
        return
      }
      const userInfo = await response.json()
      profile.id = userInfo.id
      profile.email = userInfo.email_address
    })
    return { profile }
  }
})
</script>
