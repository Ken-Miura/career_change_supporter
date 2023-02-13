<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getConsultantSideInfoDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div>
        <div v-if="!consultantSideInfo" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <div class="mt-2 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
            <div class="m-4 text-xl grid grid-cols-6 justify-center items-center">
              <div class="col-span-5">私は音声入出力テストで使用中の環境に問題がないことを確認しました</div>
              <input v-model="audioTestDone" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
            </div>
          </div>
          <button v-bind:disabled="!audioTestDone" v-on:click="processGetConsultantSideInfo" class="mt-4 col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">相談を開始する</button>
          <div v-if="getConsultantSideInfoErr.exists">
            <AlertMessage class="mt-2" v-bind:message="getConsultantSideInfoErr.message"/>
          </div>
        </div>
        <div v-else class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <div v-if="mediaError.exists">
            <AlertMessage class="mt-2" v-bind:message="mediaError.message"/>
          </div>
          <div v-else>
            <div v-if="remoteMediaStream" class="flex flex-col items-center w-full">
              <img class="w-full md:w-3/5" src="/consultant-side-consultation/user-silhouette.png" />
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
import { defineComponent, onUnmounted, reactive, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRoute, useRouter } from 'vue-router'
import { Message } from '@/util/Message'
import { useGetConsultantSideInfo } from '@/util/personalized/consultant-side-consultation/useGetConsultantSideInfo'
import { GetConsultantSideInfoResp } from '@/util/personalized/consultant-side-consultation/GetConsultantSideInfoResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { ConsultantSideInfo } from '@/util/personalized/consultant-side-consultation/ConsultantSideInfo'
import { LocalAudioStream, P2PRoom, RoomPublication, SkyWayContext, SkyWayRoom } from '@skyway-sdk/room'
import { getAudioMediaStream } from '@/util/personalized/AudioMediaStream'
import { createGetAudioMediaStreamErrMessage } from '@/util/personalized/AudioMediaStreamError'

export default defineComponent({
  name: 'ConsultantSideConsultationPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const route = useRoute()
    const consultationId = route.params.consultation_id as string
    const userAccountId = route.params.user_account_id as string
    const router = useRouter()

    const getConsultantSideInfoErr = reactive({
      exists: false,
      message: ''
    })
    const {
      getConsultantSideInfoDone,
      getConsultantSideInfoFunc
    } = useGetConsultantSideInfo()
    const audioTestDone = ref(false)
    const consultantSideInfo = ref(null as ConsultantSideInfo | null)

    const mediaError = reactive({
      exists: false,
      message: ''
    })
    let localStream = null as MediaStream | null
    let context = null as SkyWayContext | null
    let room = null as P2PRoom | null
    let localAudioStream = null as LocalAudioStream | null
    const remoteMediaStream = ref(null as MediaStream | null)

    const releaseAllResources = async () => {
      try {
        if (localAudioStream) {
          localAudioStream.release()
          localAudioStream = null
        }
      } catch (e) {
        console.error(e)
      }
      try {
        if (room) {
          room.close()
          room.dispose()
          room = null
        }
      } catch (e) {
        console.error(e)
      }
      try {
        if (context) {
          context.dispose()
          context = null
        }
      } catch (e) {
        console.error(e)
      }
      try {
        if (remoteMediaStream.value) {
          remoteMediaStream.value.getTracks().forEach(track => track.stop())
          remoteMediaStream.value = null
        }
      } catch (e) {
        console.error(e)
      }
      try {
        if (localStream) {
          localStream.getTracks().forEach(track => track.stop())
          localStream = null
        }
      } catch (e) {
        console.error(e)
      }
    }

    const processGetConsultantSideInfo = async () => {
      await releaseAllResources()
      try {
        const resp = await getConsultantSideInfoFunc(consultationId, audioTestDone.value)
        if (!(resp instanceof GetConsultantSideInfoResp)) {
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
          getConsultantSideInfoErr.exists = true
          getConsultantSideInfoErr.message = createErrorMessage(resp.getApiError().getCode())
          return
        }
        consultantSideInfo.value = resp.getConsultantSideInfo()
      } catch (e) {
        getConsultantSideInfoErr.exists = true
        getConsultantSideInfoErr.message = `${Message.UNEXPECTED_ERR}: ${e}`
        return
      }

      try {
        if (!consultantSideInfo.value) {
          mediaError.exists = true
          mediaError.message = Message.UNEXPECTED_ERR
          return
        }
        try {
          localStream = await getAudioMediaStream()
        } catch (e) {
          mediaError.exists = true
          mediaError.message = createGetAudioMediaStreamErrMessage(e)
          return
        }
        if (!localStream) {
          mediaError.exists = true
          mediaError.message = Message.FAILED_TO_GET_LOCAL_MEDIA_STREAM_ERROR_MESSAGE
          return
        }
        context = await SkyWayContext.Create(consultantSideInfo.value.token)
        if (!context) {
          mediaError.exists = true
          mediaError.message = Message.UNEXPECTED_ERR // TODO: replace error message
          return
        }
        room = await SkyWayRoom.FindOrCreate(context, {
          type: 'p2p',
          name: consultantSideInfo.value.room_name
        })
        if (!room) {
          mediaError.exists = true
          mediaError.message = Message.UNEXPECTED_ERR // TODO: replace error message
          return
        }
        const me = await room.join({ name: consultantSideInfo.value.member_name })
        localAudioStream = new LocalAudioStream(localStream.getAudioTracks()[0].clone())
        if (!localAudioStream) {
          mediaError.exists = true
          mediaError.message = Message.UNEXPECTED_ERR // TODO: replace error message
          return
        }
        me.publish(localAudioStream)

        const subscribe = async (publication: RoomPublication) => {
          if (publication.publisher.id === me.id) {
            return
          }
          const { stream } = await me.subscribe(publication.id)
          switch (stream.contentType) {
            case 'audio':
              remoteMediaStream.value = new MediaStream([stream.track.clone()])
              break
            default:
              mediaError.exists = true
              mediaError.message = Message.UNEXPECTED_ERR // TODO: replace error message
          }
        }
        room.publications.forEach(subscribe)
        room.onStreamPublished.add((e) => subscribe(e.publication))
      } catch (e) {
        mediaError.exists = true
        mediaError.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    onUnmounted(async () => {
      await releaseAllResources()
    })

    const leaveConsultationRoom = async () => {
      await router.push('/schedule')
    }

    return {
      getConsultantSideInfoErr,
      getConsultantSideInfoDone,
      audioTestDone,
      processGetConsultantSideInfo,
      consultantSideInfo,
      mediaError,
      remoteMediaStream,
      leaveConsultationRoom,
      userAccountId
    }
  }
})
</script>
