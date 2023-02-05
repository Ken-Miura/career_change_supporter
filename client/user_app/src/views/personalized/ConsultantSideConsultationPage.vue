<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getConsultantSideInfoDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="peerError.exists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="peerError.message"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <div v-if="remoteMediaStream" class="flex flex-col justify-center items-center self-center w-full md:w-3/5">
            <img class="w-full md:w-4/5 self-center" src="/user-side-consultation/consultant-silhouette.png" />
            <audio v-bind:srcObject.prop="remoteMediaStream" autoplay>
              <p class="mt-4 font-bold text-xl">使われているブラウザではサービスを利用できません。他のブラウザをお使い下さい。</p>
            </audio>
          </div>
          <div v-else>
            <h3 class="font-bold text-2xl text-center">相手が入室するまでお待ち下さい。</h3>
            <h3 class="font-bold text-2xl text-center">相手との接続が切断された場合、一度退出し、再度入室して下さい。</h3>
          </div>
          <p class="mt-6 text-xl text-center">相談時間（１時間）が過ぎたタイミングで会議室は自動的に閉じられません（相談時間が過ぎてから、一定時間後に自動的に閉じられます）相談してから１時間が経過したとき、あなたの判断で退出し、相談を終了させて下さい</p>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">相談相手の情報</h3>
        <div class="m-4 text-2xl grid grid-cols-3">
          <div class="mt-2 justify-self-start col-span-2">ユーザーID</div><div class="mt-2 justify-self-start col-span-1">{{ userAccountId }}</div>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <button v-on:click="leaveConsultationRoom" class="col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">退出する</button>
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
import { Message } from '@/util/Message'
import { useGetConsultantSideInfo } from '@/util/personalized/consultant-side-consultation/useGetConsultantSideInfo'
import { GetConsultantSideInfoResp } from '@/util/personalized/consultant-side-consultation/GetConsultantSideInfoResp'
import { closeMediaStream } from '@/util/personalized/MediaStreamCloser'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { createGetAudioMediaStreamErrMessage, getAudioMediaStream } from '@/util/personalized/AudioMediaStream'
import {
  LocalAudioStream,
  nowInSec,
  RemoteVideoStream,
  SkyWayAuthToken,
  SkyWayContext,
  SkyWayRoom,
  uuidV4
} from '@skyway-sdk/room'

export default defineComponent({
  name: 'ConsultantSideConsultationPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const appId = 'TODO'
    const secret = 'TODO'
    const token = new SkyWayAuthToken({
      jti: uuidV4(),
      iat: nowInSec(),
      exp: nowInSec() + 60 * 60 * 24,
      scope: {
        app: {
          id: appId,
          turn: true,
          actions: ['read'],
          channels: [
            {
              id: '*',
              name: '*',
              actions: ['write'],
              members: [
                {
                  id: '*',
                  name: '*',
                  actions: ['write'],
                  publication: {
                    actions: ['write']
                  },
                  subscription: {
                    actions: ['write']
                  }
                }
              ],
              sfuBots: [
                {
                  actions: ['write'],
                  forwardings: [
                    {
                      actions: ['write']
                    }
                  ]
                }
              ]
            }
          ]
        }
      }
    }).encode(secret)
    const roomName = '6ebb6af49c654d10a41babbe60251baf'

    const route = useRoute()
    const consultationId = route.params.consultation_id as string
    const userAccountId = route.params.user_account_id as string

    const {
      getConsultantSideInfoDone,
      getConsultantSideInfoFunc
    } = useGetConsultantSideInfo()

    const peerError = reactive({
      exists: false,
      message: ''
    })
    const remoteMediaStream = ref(null as MediaStream | null)

    let localStream = null as MediaStream | null

    const router = useRouter()

    onMounted(async () => {
      try {
        try {
          localStream = await getAudioMediaStream()
        } catch (e) {
          peerError.exists = true
          peerError.message = createGetAudioMediaStreamErrMessage(e)
          return
        }
        if (!localStream) {
          peerError.exists = true
          peerError.message = Message.FAILED_TO_GET_LOCAL_MEDIA_STREAM_ERROR_MESSAGE
          return
        }

        const context = await SkyWayContext.Create(token)
        const room = await SkyWayRoom.FindOrCreate(context, {
          type: 'p2p',
          name: roomName
        })

        const me = await room.join()
        const audioTracks = localStream.getAudioTracks()
        const localAudioStream = new LocalAudioStream(audioTracks[0].clone())
        await me.publish(localAudioStream)
        // eslint-disable-next-line
        const subscribeAndAttach = async (publication: any) => {
          if (publication.publisher.id === me.id) {
            return
          }
          const { stream } = await me.subscribe<RemoteVideoStream>(
            publication.id
          )
          switch (stream.track.kind) {
            case 'video':
              console.error('video')
              break
            case 'audio':
              remoteMediaStream.value = new MediaStream([stream.track])
              break
            default:
              console.error('default')
          }
        }
        room.publications.forEach(subscribeAndAttach)
        room.onStreamPublished.add((e) => subscribeAndAttach(e.publication))
      } catch (e) {
        peerError.exists = true
        peerError.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    onUnmounted(() => {
      closeMediaStream(localStream)
      closeMediaStream(remoteMediaStream.value)
    })

    const leaveConsultationRoom = async () => {
      await router.push('/schedule')
    }

    return {
      getConsultantSideInfoDone,
      peerError,
      remoteMediaStream,
      leaveConsultationRoom,
      userAccountId
    }
  }
})
</script>
