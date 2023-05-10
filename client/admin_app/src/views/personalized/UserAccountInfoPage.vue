<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="false" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      {{ testMessage }}
      <div v-if="errorMessage">
        <AlertMessage class="mt-6" v-bind:message="errorMessage"/>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex flex-col text-white">
      <router-link to="/admin-menu" class="hover:underline text-center">管理メニューへ</router-link>
      <router-link to="/" class="mt-6 hover:underline text-center">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useStore } from 'vuex'
import { UserAccountSearchParam } from '@/util/personalized/user-account-search/UserAccountSearchParam'

export default defineComponent({
  name: 'UserAccountInfoPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const testMessage = ref('')
    const store = useStore()

    const errorMessage = ref(null as string | null)

    onMounted(async () => {
      const param = store.state.userAccountSearchParam as UserAccountSearchParam
      if (!param) {
        testMessage.value = '!param'
        return
      }
      testMessage.value = `${param.accountId}, ${param.emailAddress}`
    })

    return {
      testMessage,
      errorMessage
    }
  }
})
</script>
