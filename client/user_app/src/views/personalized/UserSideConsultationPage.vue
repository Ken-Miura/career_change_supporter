<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg text-center">{{ message }}</h3>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import { useRoute } from 'vue-router'
import { getSkyWayApiKey } from '@/util/SkyWay'
import { useGetUserSideInfo } from '@/util/personalized/user-side-consultation/useGetUserSideInfo'
import { GetUserSideInfoResp } from '@/util/personalized/user-side-consultation/GetUserSideInfoResp'
import Peer from 'skyway-js'

export default defineComponent({
  name: 'UserSideConsultationPage',
  components: {
    TheHeader
  },
  setup () {
    const skyWayApiKey = getSkyWayApiKey()
    const route = useRoute()
    const consultationId = route.params.consultation_id as string
    const message = `UserSideConsultationPage ${consultationId} ${skyWayApiKey}`

    const {
      getUserSideInfoDone,
      getUserSideInfoFunc
    } = useGetUserSideInfo()

    onMounted(async () => {
      console.log(getUserSideInfoDone.value)
      try {
        const response = await getUserSideInfoFunc(consultationId)
        if (response instanceof GetUserSideInfoResp) {
          const result = response.getUserSideInfo()
          console.log(result)
          const peer = new Peer(result.user_account_peer_id, { key: skyWayApiKey, credential: result.credential, debug: 3 })
          peer.on('open', function () {
            console.log('peer open')
          })
        }
      } catch (e) {
        console.log(e)
      }
    })

    return { message }
  }
})
</script>
