<template>
  <div id="nav">
    <router-link to="/">ホーム</router-link> |
    <div v-if="!loggedIn">
      <router-link to="/login">ログイン</router-link> |
      <router-link to="/register">アカウント作成</router-link>
    </div>
    <div v-if="loggedIn">
      <router-link to="/schedule">スケジュール</router-link> |
      <router-link to="/profile">プロファイル</router-link> |
      <router-link to="/search">検索</router-link> |
      <!-- TODO: Adjust touchable area regarding logout -->
      <u class="logout"><b><div @click="logout">ログアウト</div></b></u>
    </div>
  </div>
  <router-view/>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted, onUnmounted } from 'vue'
import { useStore } from 'vuex'
import { useRouter } from 'vue-router'
import { getSessionState } from '@/store/SessionChecker'

export default defineComponent({
  name: 'App',
  setup () {
    const loggedIn = ref(false)
    const store = useStore()
    const unwatch = store.watch(
      (state) => state.sessionState,
      // Add special commnet to suppress unused parameter
      // eslint-disable-next-line
      (newValue, _oldValue) => { loggedIn.value = (newValue === 'active') })
    onMounted(async () => {
      const sessionState = await getSessionState()
      store.commit('updateSessionState', sessionState)
    })
    onUnmounted(() => { unwatch() })
    const router = useRouter()
    const logout = async () => {
      store.commit('updateSessionState', 'none')
      const response = await fetch('logout-request', {
        method: 'POST'
      })
      if (!response.ok) {
        const text = await response.text()
        console.log('failed logout request. status: %d, body: %s', response.status, text)
      }
      const redirectToLogin = async () => await router.push('login')
      redirectToLogin()
    }
    return {
      loggedIn,
      logout
    }
  }
})
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
}

#nav {
  padding: 30px;
}

#nav a {
  font-weight: bold;
  color: #2c3e50;
}

#nav a.router-link-exact-active {
  color: #42b983;
}

.logout {
  cursor: pointer;
}
</style>
