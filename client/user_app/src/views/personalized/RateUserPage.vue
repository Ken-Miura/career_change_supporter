<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!postUserRatingDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 data-test="user-rating-label" class="font-bold text-2xl">相談を受け付けたユーザーの評価</h3>
        <p data-test="user-rating-description" class="mt-4 ml-2 text-xl">相談を行ったユーザーを評価して下さい。{{ MIN_RATING }}が最も低い（悪い）評価で、{{ MAX_RATING }}が最も高い（良い）評価となります。</p>
        <div class="mt-2 ml-4 grid grid-cols-3">
          <p data-test="user-id-label" class="mt-4 justify-self-start text-xl col-span-2">ユーザーID</p>
          <p data-test="user-id-value" class="mt-4 justify-self-center text-xl col-span-1">{{ userId }}</p>
          <p data-test="consultation-date-time-label" class="mt-4 justify-self-start text-xl col-span-2">相談実施日時</p>
          <p data-test="consultation-date-time-value" class="mt-4 justify-self-center text-xl col-span-1">{{ year }}年{{ month }}月{{ day }}日{{ hour }}時</p>
          <p data-test="rating-label" class="mt-4 justify-self-start text-xl col-span-2">評価</p>
          <p data-test="rating-value" class="mt-4 justify-self-center text-xl w-full col-span-1">
            <select v-model="rating" class="block w-full p-3 text-center rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
              <option value=""></option>
              <option value="5">5</option>
              <option value="4">4</option>
              <option value="3">3</option>
              <option value="2">2</option>
              <option value="1">1</option>
            </select>
          </p>
        </div>
        <button data-test="submit-button" v-on:click="submitRating" v-bind:disabled="!rating" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">評価する</button>
        <div v-if="errMessage">
          <AlertMessage class="mt-6" v-bind:message="errMessage"/>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRoute, useRouter } from 'vue-router'
import { usePostUserRating } from '@/util/personalized/rate-user/usePostUserRating'
import { MAX_RATING, MIN_RATING } from '@/util/personalized/RatingConstants'
import { Message } from '@/util/Message'
import { Code, createErrorMessage } from '@/util/Error'
import { ApiErrorResp } from '@/util/ApiError'
import { PostUserRatingResp } from '@/util/personalized/rate-user/PostUserRatingResp'

export default defineComponent({
  name: 'RateUserPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()
    const route = useRoute()
    const userRatingId = route.params.user_rating_id as string
    const query = route.query
    const userId = query['user-id']
    const year = query.year
    const month = query.month
    const day = query.day
    const hour = query.hour

    const {
      postUserRatingDone,
      postUserRatingFunc
    } = usePostUserRating()

    const errMessage = ref(null as string | null)

    const rating = ref('' as string)

    const submitRating = async () => {
      try {
        const r = parseInt(rating.value)
        if (!(MIN_RATING <= r && r <= MAX_RATING)) {
          errMessage.value = Message.INVALID_RATING_MESSAGE
          return
        }
        const resp = await postUserRatingFunc(parseInt(userRatingId), r)
        if (!(resp instanceof PostUserRatingResp)) {
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
          errMessage.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
        await router.push('/rate-success')
      } catch (e) {
        errMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    return {
      postUserRatingDone,
      errMessage,
      MIN_RATING,
      MAX_RATING,
      userId,
      year,
      month,
      day,
      hour,
      rating,
      submitRating
    }
  }
})
</script>
