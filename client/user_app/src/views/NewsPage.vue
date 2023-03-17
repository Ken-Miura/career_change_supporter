<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <div>
      <div v-if="!getNewsDone" class="m-6">
        <WaitingCircle />
      </div>
      <main v-else>
        <div v-if="!errMessage" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 data-test="news-label" class="font-bold text-2xl">お知らせ</h3>
          <div data-test="no-news-found" class="mt-6 ml-2 text-xl" v-if="newsArray.length === 0">お知らせはありません</div>
          <div v-else>
            <ul>
              <li v-for="news in newsArray" v-bind:key="news.news_id">
                <div class="mt-6 ml-2">
                  <h2 class="font-bold text-xl">{{ news.published_date_in_jst.year }}年{{ news.published_date_in_jst.month }}月{{ news.published_date_in_jst.day }}日 【{{ news.title }}】</h2>
                  <p class="mt-2 ml-4 text-xl whitespace-pre-wrap">{{ news.body }}</p>
                </div>
              </li>
            </ul>
          </div>
        </div>
        <div v-else class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage v-bind:message="errMessage"/>
        </div>
      </main>
    </div>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useGetNews } from '@/util/news/useGetNews'
import { News } from '@/util/news/News'
import { Message } from '@/util/Message'
import { GetNewResp } from '@/util/news/GetNewResp'
import { ApiErrorResp } from '@/util/ApiError'
import { createErrorMessage } from '@/util/Error'

export default defineComponent({
  name: 'NewsPage',
  components: {
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const {
      getNewsDone,
      getNewsFunc
    } = useGetNews()

    const errMessage = ref(null as string | null)

    const newsArray = ref([] as News[])

    onMounted(async () => {
      try {
        const resp = await getNewsFunc()
        if (!(resp instanceof GetNewResp)) {
          if (!(resp instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${resp}`)
          }
          errMessage.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
        newsArray.value = resp.getNewsResult().news_array
      } catch (e) {
        errMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    return {
      getNewsDone,
      errMessage,
      newsArray
    }
  }
})
</script>
