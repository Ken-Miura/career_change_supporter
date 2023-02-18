<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!refreshDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <div v-if="refreshErrorMessage">
        <AlertMessage class="mt-2" v-bind:message="refreshErrorMessage"/>
      </div>
      <div v-else>
        <h3 class="font-bold text-2xl">音声入出力テスト</h3>
        <p class="mt-4 ml-2 text-xl">あなたがお使いの環境で相談を実施可能かテストを行うことが出来ます（※1）下記の「音声入出力テストを開始」を押し、何かマイクに向けて話しかけて下さい。加工された音声（普段のあなたの声より高い、または低い声）（※2）が聞こえれば、あなたがお使いの環境で相談を実施可能です。</p>
        <p class="mt-2 ml-2">（※1）音声入出力に関する内容のみがテスト対象です。相談を行う前に通信環境が問題ないことは別途ご確認下さい。</p>
        <p class="ml-2">（※2）（音声入出力テスト開始時、相談開始時のどちらの場合でも）声の高さの変化具合はランダムに決まります。もし、加工後の音声が聞き取りづらい場合、「音声入出力テストを停止」を押し、その後、再度「音声入出力テストを開始」を押して下さい。</p>
        <div class="mt-4 ml-4">
          <div v-if="audioTestErrorMessage">
            <AlertMessage class="mt-2" v-bind:message="audioTestErrorMessage"/>
          </div>
          <div class="flex flex-col" v-else>
            <button v-bind:disabled="audioTestStarted" v-on:click="startAudioTest" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">音声入出力テストを開始</button>
            <button v-bind:disabled="!audioTestStarted" v-on:click="stopAudioTest" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">音声入出力テストを停止</button>
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
import { useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRefresh } from '@/util/personalized/refresh/useRefresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { Message } from '@/util/Message'
import { ProcessedAudioConnectedWithSpeaker } from '@/util/personalized/processed-audio/ProcessedAudio'
import { ProcessedAudioError } from '@/util/personalized/processed-audio/ProcessedAudioError'

export default defineComponent({
  name: 'AudioTestPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()

    const {
      refreshDone,
      refreshFunc
    } = useRefresh()
    const refreshErrorMessage = ref(null as string | null)

    const audioTestStarted = ref(false)
    const audioTestErrorMessage = ref(null as string | null)
    let processedAudioConnectedWithSpeaker: ProcessedAudioConnectedWithSpeaker | null

    onMounted(async () => {
      try {
        const resp = await refreshFunc()
        if (!(resp instanceof RefreshResp)) {
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
          refreshErrorMessage.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
      } catch (e) {
        refreshErrorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const releaseAudioResouces = async () => {
      if (processedAudioConnectedWithSpeaker) {
        await processedAudioConnectedWithSpeaker.close()
        processedAudioConnectedWithSpeaker = null
      }
    }

    const startAudioTest = async () => {
      audioTestStarted.value = true
      try {
        const p = new ProcessedAudioConnectedWithSpeaker()
        processedAudioConnectedWithSpeaker = p
        await p.init()
      } catch (e) {
        if (e instanceof ProcessedAudioError) {
          audioTestErrorMessage.value = `${e.message}`
        } else {
          audioTestErrorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
        }
        await releaseAudioResouces()
      }
    }

    const stopAudioTest = async () => {
      await releaseAudioResouces()
      audioTestStarted.value = false
    }

    onUnmounted(async () => {
      await releaseAudioResouces()
    })

    return {
      refreshDone,
      refreshErrorMessage,
      audioTestErrorMessage,
      startAudioTest,
      stopAudioTest,
      audioTestStarted
    }
  }
})
</script>
