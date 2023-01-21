<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getUserSideInfoDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="error.exists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="error.message"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">{{ message }}</h3>
          <div v-if="mediaStream">
            <video v-bind:srcObject.prop="mediaStream" autoplay playsinline></video>
          </div>
          <div v-else>
            <p>mediaStream === null</p>
          </div>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, onUnmounted, reactive, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRoute } from 'vue-router'
import { getSkyWayApiKey } from '@/util/SkyWay'
import { useGetUserSideInfo } from '@/util/personalized/user-side-consultation/useGetUserSideInfo'
import { GetUserSideInfoResp } from '@/util/personalized/user-side-consultation/GetUserSideInfoResp'
import Peer from 'skyway-js'
import { Message } from '@/util/Message'
import { ApiErrorResp } from '@/util/ApiError'
import { createErrorMessage } from '@/util/Error'

export default defineComponent({
  name: 'UserSideConsultationPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
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

    const mediaStream = ref(null as MediaStream | null)
    let peer = null as Peer | null

    const error = reactive({
      exists: false,
      message: ''
    })

    onMounted(async () => {
      try {
        const resp = await getUserSideInfoFunc(consultationId)
        if (!(resp instanceof GetUserSideInfoResp)) {
          if (!(resp instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${resp}`)
          }
          // const code = resp.getApiError().getCode()
          // if (code === Code.UNAUTHORIZED) {
          //   await router.push('/login')
          //   return
          // } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
          //   await router.push('/terms-of-use')
          //   return
          // }
          error.exists = true
          error.message = createErrorMessage(resp.getApiError().getCode())
          return
        }
        const result = resp.getUserSideInfo()

        const localStream = await navigator.mediaDevices
          .getUserMedia({
            audio: true,
            video: true
          })

        peer = new Peer(result.user_account_peer_id, { key: skyWayApiKey, credential: result.credential, debug: 0 })
        if (!peer) {
          console.log('!peer')
          return
        }

        peer.on('error', e => {
          error.exists = true
          error.message = `${Message.UNEXPECTED_ERR}: ${e}`
        })

        peer.on('call', (mediaConnection) => {
          console.log('on call')

          mediaConnection.answer(localStream)

          mediaConnection.on('stream', async stream => {
            console.log('stream event on callee side')
            mediaStream.value = stream
          })

          mediaConnection.once('close', () => {
            console.log('close event on callee side')
            if (!mediaStream.value) {
              return
            }
            const ms = mediaStream.value
            ms.getTracks().forEach(track => track.stop())
            mediaStream.value = null
          })
        })

        // const consultantPeerId = result.consultant_peer_id
        // if (!consultantPeerId) {
        //   console.log('!consultantPeerId')
        //   return
        // }
        // console.log('consultantPeerId: ' + consultantPeerId)
        peer.on('open', async function () {
          console.log('peer open')
          // if (!peer) {
          //   console.log('!peer')
          //   return
          // }

          // const mediaConnection = peer.call(consultantPeerId, localStream)

          // mediaConnection.on('stream', async stream => {
          //   console.log('stream event on caller side')
          //   mediaStream.value = stream
          // })

          // mediaConnection.once('close', () => {
          //   if (!mediaStream.value) {
          //     console.log('close event on caller side')
          //     return
          //   }
          //   mediaStream.value.getTracks().forEach(track => track.stop())
          //   mediaStream.value = null
          // })
        })
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    onUnmounted(async () => {
      if (!peer) {
        console.log('!peer')
        return
      }
      peer.destroy()
      peer = null
    })

    return {
      getUserSideInfoDone,
      error,
      message,
      mediaStream
    }
  }
})
</script>
