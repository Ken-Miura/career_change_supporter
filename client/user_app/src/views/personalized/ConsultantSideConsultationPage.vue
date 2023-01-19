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
          <div>
            <video id="consultant-side-sky-way-remote-stream"></video>
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
import Peer, { MediaConnection } from 'skyway-js'
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

    const peer = ref(null as Peer | null)
    const mediaConnection = ref(null as MediaConnection | null)

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

          const remoteVideo = document.getElementById('consultant-side-sky-way-remote-stream')
          if (!remoteVideo) {
            console.log('!remoteVideo')
            return
          }
          if (!(remoteVideo instanceof HTMLVideoElement)) {
            console.log('!(remoteVideo instanceof HTMLVideoElement)')
            return
          }
          const localStream = await navigator.mediaDevices
            .getUserMedia({
              audio: true,
              video: true
            })

          peer.value = new Peer(result.consultant_peer_id, { key: skyWayApiKey, credential: result.credential, debug: 3 })
          if (!peer.value) {
            console.log('!peer.value')
            return
          }

          peer.value.on('error', e => {
            error.exists = true
            error.message = `${Message.UNEXPECTED_ERR}: ${e}`
          })

          peer.value.on('call', (mc) => {
            if (!mediaConnection.value) {
              console.log('!mediaConnection.value')
              return
            }
            mediaConnection.value = mc
            mediaConnection.value.answer(localStream)

            mediaConnection.value.on('stream', async stream => {
              remoteVideo.srcObject = stream
              remoteVideo.playsInline = true
              try {
                await remoteVideo.play()
              } catch (e) {
                error.exists = true
                error.message = `${Message.UNEXPECTED_ERR}: ${e}`
              }
            })

            mediaConnection.value.once('close', () => {
              const srcObj = remoteVideo.srcObject
              if (!srcObj) {
                console.log('!srcObj')
                return
              }
              if (!(srcObj instanceof MediaStream)) {
                console.log('!(srcObj instanceof MediaStream)')
                return
              }
              srcObj.getTracks().forEach(track => track.stop())
              remoteVideo.srcObject = null
            })
          })

          const userAccountPeerId = result.user_account_peer_id
          if (!userAccountPeerId) {
            console.log('!userAccountPeerId')
            return
          }
          peer.value.on('open', async function () {
            if (!mediaConnection.value) {
              console.log('!mediaConnection.value')
              return
            }
            if (!peer.value) {
              console.log('!peer.value')
              return
            }
            const mc = peer.value.call(userAccountPeerId, localStream)
            mediaConnection.value = mc

            mediaConnection.value.on('stream', async stream => {
              remoteVideo.srcObject = stream
              remoteVideo.playsInline = true
            })
            try {
              await remoteVideo.play()
            } catch (e) {
              error.exists = true
              error.message = `${Message.UNEXPECTED_ERR}: ${e}`
            }

            mediaConnection.value.once('close', () => {
              const srcObj = remoteVideo.srcObject
              if (!srcObj) {
                console.log('!srcObj')
                return
              }
              if (!(srcObj instanceof MediaStream)) {
                console.log('!(srcObj instanceof MediaStream)')
                return
              }
              srcObj.getTracks().forEach(track => track.stop())
              remoteVideo.srcObject = null
            })
          })
        }
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    onUnmounted(async () => {
      if (!mediaConnection.value) {
        console.log('!mediaConnection.value')
      } else {
        mediaConnection.value.close(true)
        mediaConnection.value = null
      }
      if (!peer.value) {
        console.log('!peer.value')
      } else {
        peer.value.destroy()
        peer.value = null
      }
    })

    return {
      getConsultantSideInfoDone,
      error,
      message
    }
  }
})
</script>
