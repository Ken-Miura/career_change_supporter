<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!(getLatestNewsDone && postSetNewsReqDone)" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">お知らせの作成</h3>
        <form @submit.prevent="setNews">
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-2 text-2xl justify-self-start col-span-6 pt-3">
              タイトル
            </div>
            <div class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <input v-model="title" type="text" required minlength="1" maxlength="256" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              本文
            </div>
            <div class="mt-2 min-w-full justify-self-start col-span-6 pt-3 rounded bg-gray-200">
              <textarea v-model="body" minlength="1" maxlength="16384" placeholder="お知らせ本文" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3"></textarea>
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              プレビュー
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              タイトル
            </div>
            <div class="mt-2 ml-2 min-w-full justify-self-start col-span-6 pt-3">
              {{ title }}
            </div>
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              本文
            </div>
            <div class="mt-2 ml-2 min-w-full justify-self-start col-span-6 pt-3 whitespace-pre-wrap">
              {{ body }}
            </div>
          </div>
          <div class="mt-6 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
            <div class="p-4 text-xl grid grid-cols-6 justify-center items-center">
              <div class="col-span-5">お知らせの内容、表示が適正であることを確認しました</div>
              <input v-model="setNewsConfirmation" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
            </div>
          </div>
          <button v-bind:disabled="!setNewsConfirmation" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none" type="submit">お知らせを作成する</button>
          <div v-if="setNewsErrMessage" class="mt-6">
            <AlertMessage v-bind:message="setNewsErrMessage"/>
          </div>
        </form>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">お知らせ一覧</h3>
        <div v-if="!latestNewsErrMessage">
            <div v-if="latestNews.length !== 0">
              <ul>
                <li v-for="n in latestNews" v-bind:key="n.news_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">お知らせ番号{{ n.news_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div class="mt-2 justify-self-start col-span-1">掲載日時</div><div class="mt-2 justify-self-start col-span-2">{{ n.published_at }}</div>
                    <div class="mt-2 justify-self-start col-span-1">タイトル</div><div class="mt-2 justify-self-start col-span-2">{{ n.title }}</div>
                    <div class="mt-2 justify-self-start col-span-1">本文</div><div class="mt-2 justify-self-start col-span-2 whitespace-pre-wrap">{{ n.body }}</div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              お知らせはありません。
            </div>
          </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="latestNewsErrMessage"/>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex flex-col text-white">
      <router-link to="/admin-menu" class="hover:underline text-center">管理メニューへ</router-link>
      <router-link to="/" class="mt-6 hover:underline text-center">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRouter } from 'vue-router'
import { Code, createErrorMessage } from '@/util/Error'
import { ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'
import { News } from '@/util/personalized/latest-news/News'
import { useGetLatestNews } from '@/util/personalized/latest-news/useGetLatestNews'
import { GetLatestNewsResp } from '@/util/personalized/latest-news/GetLatestNewsResp'
import { usePostSetNewsReq } from '@/util/personalized/set-news-req/usePostSetNewsReq'
import { SetNewsReq } from '@/util/personalized/set-news-req/SetNewsReq'
import { PostSetNewsReqResp } from '@/util/personalized/set-news-req/PostSetNewsReqResp'

export default defineComponent({
  name: 'NewsPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()

    const latestNews = ref([] as News[])
    const latestNewsErrMessage = ref(null as string | null)

    const {
      getLatestNewsDone,
      getLatestNewsFunc
    } = useGetLatestNews()

    const getLatestNews = async () => {
      try {
        const response = await getLatestNewsFunc()
        if (!(response instanceof GetLatestNewsResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          latestNewsErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getLatestNewsResult()
        latestNews.value = result.news_array
        latestNewsErrMessage.value = null
      } catch (e) {
        latestNewsErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    onMounted(async () => {
      await getLatestNews()
    })

    const title = ref('')
    const body = ref('')
    const setNewsErrMessage = ref(null as string | null)

    const {
      postSetNewsReqDone,
      postSetNewsReqFunc
    } = usePostSetNewsReq()

    const setNewsConfirmation = ref(false)

    const setNews = async () => {
      const req = {
        title: title.value,
        body: body.value
      } as SetNewsReq
      try {
        const response = await postSetNewsReqFunc(req)
        if (!(response instanceof PostSetNewsReqResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          setNewsErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        setNewsErrMessage.value = null
        await getLatestNews()
      } catch (e) {
        setNewsErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      } finally {
        setNewsConfirmation.value = false
      }
    }

    return {
      getLatestNewsDone,
      latestNews,
      latestNewsErrMessage,
      title,
      body,
      setNews,
      setNewsErrMessage,
      postSetNewsReqDone,
      setNewsConfirmation
    }
  }
})
</script>
