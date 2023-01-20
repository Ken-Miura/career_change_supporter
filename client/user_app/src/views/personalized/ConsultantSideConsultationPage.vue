<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getConsultantSideInfoDone" class="m-6">
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
import { Message } from '@/util/Message'
import Peer from 'skyway-js'
import { useGetConsultantSideInfo } from '@/util/personalized/consultant-side-consultation/useGetConsultantSideInfo'
import { GetConsultantSideInfoResp } from '@/util/personalized/consultant-side-consultation/GetConsultantSideInfoResp'
import { ApiErrorResp } from '@/util/ApiError'
import { createErrorMessage } from '@/util/Error'

export default defineComponent({
  name: 'ConsultantSideConsultationPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const skyWayApiKey = getSkyWayApiKey()
    const route = useRoute()
    const consultationId = route.params.consultation_id as string
    const message = `ConsultantSideConsultationPage ${consultationId} ${skyWayApiKey}`

    const {
      getConsultantSideInfoDone,
      getConsultantSideInfoFunc
    } = useGetConsultantSideInfo()

    const mediaStream = ref(null as MediaStream | null)
    let peer = null as Peer | null

    const error = reactive({
      exists: false,
      message: ''
    })

    onMounted(async () => {
      try {
        const response = await getConsultantSideInfoFunc(consultationId)
        if (response instanceof ApiErrorResp) {
          error.exists = true
          error.message = createErrorMessage(response.getApiError().getCode())
          return
        }
        if (response instanceof GetConsultantSideInfoResp) {
          const result = response.getConsultantSideInfo()

          const localStream = await navigator.mediaDevices
            .getUserMedia({
              audio: true,
              video: true
            })

          peer = new Peer(result.consultant_peer_id, { key: skyWayApiKey, credential: result.credential, debug: 0 })
          if (!peer) {
            console.log('!peer')
            return
          }

          peer.on('error', e => {
            error.exists = true
            error.message = `${Message.UNEXPECTED_ERR}: ${e}`
          })

          peer.on('call', (mediaConnection) => {
            if (!localStream) {
              console.log('!localStream')
              return
            }
            mediaConnection.answer(localStream)

            mediaConnection.on('stream', async stream => {
              console.log('mediaStream.value = stream 1')
              mediaStream.value = stream
            })

            mediaConnection.once('close', () => {
              if (!mediaStream.value) {
                console.log('!mediaStream.value 1')
                return
              }
              mediaStream.value.getTracks().forEach(track => track.stop())
              mediaStream.value = null
            })
          })

          const userAccountPeerId = result.user_account_peer_id
          if (!userAccountPeerId) {
            console.log('!userAccountPeerId')
            return
          }
          console.log('userAccountPeerId: ' + userAccountPeerId)
          peer.on('open', async function () {
            if (!peer) {
              console.log('!peer')
              return
            }
            if (!userAccountPeerId) {
              console.log('!userAccountPeerId')
              return
            }
            if (!localStream) {
              console.log('!localStream')
              return
            }
            const mediaConnection = peer.call(userAccountPeerId, localStream)

            mediaConnection.on('stream', async stream => {
              console.log('mediaStream.value = stream 2')
              mediaStream.value = stream
            })

            mediaConnection.once('close', () => {
              if (!mediaStream.value) {
                console.log('!mediaStream.value 2')
                return
              }
              mediaStream.value.getTracks().forEach(track => track.stop())
              mediaStream.value = null
            })
          })
        }
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    onUnmounted(async () => {
      if (!peer) {
        console.log('!peer')
      } else {
        peer.destroy()
        peer = null
      }
    })

    return {
      getConsultantSideInfoDone,
      error,
      message,
      mediaStream
    }
  }
})
</script>
