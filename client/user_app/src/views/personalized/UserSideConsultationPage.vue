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
        <div class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <div v-if="mediaStream" class="flex flex-col justify-center items-center self-center w-full md:w-3/5">
            <img class="w-full md:w-4/5 z-50 self-center" src="/user-side-consultation/consultant-silhouette.png" />
            <audio v-bind:srcObject.prop="mediaStream" autoplay>
              <p class="mt-4 font-bold text-xl">使われているブラウザではサービスを利用できません。他のブラウザをお使い下さい。</p>
            </audio>
          </div>
          <div v-else>
            <h3 class="font-bold text-2xl text-center">相手が入室するまでお待ち下さい</h3>
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
import { useRoute, useRouter } from 'vue-router'
import { getSkyWayApiKey } from '@/util/SkyWay'
import { useGetUserSideInfo } from '@/util/personalized/user-side-consultation/useGetUserSideInfo'
import { GetUserSideInfoResp } from '@/util/personalized/user-side-consultation/GetUserSideInfoResp'
import Peer from 'skyway-js'
import { Message } from '@/util/Message'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'

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

    const {
      getUserSideInfoDone,
      getUserSideInfoFunc
    } = useGetUserSideInfo()

    const mediaStream = ref(null as MediaStream | null)
    let peer = null as Peer | null
    let localStream = null as MediaStream | null

    const error = reactive({
      exists: false,
      message: ''
    })

    const router = useRouter()

    onMounted(async () => {
      try {
        const resp = await getUserSideInfoFunc(consultationId)
        if (!(resp instanceof GetUserSideInfoResp)) {
          if (!(resp instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${resp}`)
          }
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          }
          error.exists = true
          error.message = createErrorMessage(resp.getApiError().getCode())
          return
        }
        const result = resp.getUserSideInfo()

        localStream = await window.navigator.mediaDevices
          .getUserMedia({
            audio: true,
            video: false
          })
        if (!localStream) {
          error.exists = true
          error.message = '!localStream'
          return
        }

        peer = new Peer(result.user_account_peer_id, { key: skyWayApiKey, credential: result.credential, debug: 0 })
        if (!peer) {
          error.exists = true
          error.message = '!peer'
          return
        }

        peer.on('error', e => {
          const errType = e.type
          // fetchPeerExistsを行わずにcallを行うので発生が予見されるエラー
          // そのため、特に何もしない（一度お互いに入室し、その後何らかの理由で再度入室することになった場合発生し得る）
          if (errType === 'peer-unavailable') {
            return
          }
          error.exists = true
          error.message = `${Message.UNEXPECTED_ERR}: ${e} ${errType}`
        })

        peer.on('call', (mediaConnection) => {
          if (!localStream) {
            error.exists = true
            error.message = '!localStream'
            return
          }
          mediaConnection.answer(localStream)

          mediaConnection.on('stream', async stream => {
            mediaStream.value = stream
          })

          mediaConnection.once('close', () => {
            if (!mediaStream.value) {
              return
            }
            mediaStream.value.getTracks().forEach(track => track.stop())
            mediaStream.value = null
          })
        })

        const consultantPeerId = result.consultant_peer_id
        if (!consultantPeerId) {
          error.exists = true
          error.message = '!consultantPeerId'
          return
        }
        peer.on('open', async function () {
          if (!peer) {
            error.exists = true
            error.message = '!peer'
            return
          }
          if (!localStream) {
            error.exists = true
            error.message = '!localStream'
            return
          }

          // fetchPeerExistsで事前に確認してから通信したほうが確実だが
          // rate limitが厳しすぎるので使わない
          const mediaConnection = peer.call(consultantPeerId, localStream)

          mediaConnection.on('stream', async stream => {
            mediaStream.value = stream
          })

          mediaConnection.once('close', () => {
            if (!mediaStream.value) {
              return
            }
            mediaStream.value.getTracks().forEach(track => track.stop())
            mediaStream.value = null
          })
        })
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const closePeer = () => {
      try {
        if (peer) {
          peer.destroy()
          peer = null
        }
      } catch (e) {
        console.error(e)
      }
    }

    const closeLocalStream = () => {
      try {
        if (localStream) {
          localStream.getTracks().forEach(track => track.stop())
          localStream = null
        }
      } catch (e) {
        console.error(e)
      }
    }

    onUnmounted(async () => {
      closePeer()
      closeLocalStream()
    })

    return {
      getUserSideInfoDone,
      error,
      mediaStream
    }
  }
})
</script>
