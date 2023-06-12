<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!requestsDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">相談</h3>
        <div v-if="!consultationErrMessage">
          <div v-if="consultation" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ consultation.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">コンサルタントID</div><div class="mt-2 justify-self-start col-span-4">{{ consultation.consultant_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">ユーザーアカウントID</div><div class="mt-2 justify-self-start col-span-4">{{ consultation.user_account_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談日時</div><div class="mt-2 justify-self-start col-span-4">{{ consultation.meeting_at }}</div>
            <div class="mt-2 justify-self-start col-span-3">部屋名</div><div class="mt-2 justify-self-start col-span-4">{{ consultation.room_name }}</div>
            <div class="mt-2 justify-self-start col-span-3">ユーザー入室日時</div><div v-if="consultation.user_account_entered_at" class="mt-2 justify-self-start col-span-4">{{ consultation.user_account_entered_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">入室記録なし</div>
            <div class="mt-2 justify-self-start col-span-3">コンサルタント入室日時</div><div v-if="consultation.consultant_entered_at" class="mt-2 justify-self-start col-span-4">{{ consultation.consultant_entered_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">入室記録なし</div>
          </div>
          <div v-else class="m-4 text-2xl">
            相談は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="consultationErrMessage"/>
        </div>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">コンサルタントからのユーザーに対する評価</h3>
        <div v-if="!userRatingErrMessage">
          <div v-if="userRating" class="m-4 text-2xl grid grid-cols-7">
            <div class="mt-2 justify-self-start col-span-3">ユーザー評価番号</div><div class="mt-2 justify-self-start col-span-4">{{ userRating.user_rating_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">相談番号</div><div class="mt-2 justify-self-start col-span-4">{{ userRating.consultation_id }}</div>
            <div class="mt-2 justify-self-start col-span-3">評価</div><div v-if="userRating.rating" class="mt-2 justify-self-start col-span-4">{{ userRating.rating }}</div><div v-else class="mt-2 justify-self-start col-span-4">未評価</div>
            <div class="mt-2 justify-self-start col-span-3">評価日時</div><div v-if="userRating.rated_at" class="mt-2 justify-self-start col-span-4">{{ userRating.rated_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">未評価</div>
          </div>
          <div v-else class="m-4 text-2xl">
            コンサルタントからのユーザーに対する評価は見つかりませんでした
          </div>
        </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="userRatingErrMessage"/>
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
import { defineComponent, ref, onMounted, computed } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRoute, useRouter } from 'vue-router'
import { Consultation } from '@/util/personalized/Consultation'
import { useGetConsultationByConsultationId } from '@/util/personalized/consultation/useGetConsultationByConsultationId'
import { Message } from '@/util/Message'
import { GetConsultationByConsultationIdResp } from '@/util/personalized/consultation/GetConsultationByConsultationIdResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { useGetUserRatingByConsultationId } from '@/util/personalized/consultation/user-rating/useGetUserRatingByConsultationId'
import { UserRating } from '@/util/personalized/consultation/user-rating/UserRating'
import { GetUserRatingByConsultationIdResp } from '@/util/personalized/consultation/user-rating/GetUserRatingByConsultationIdResp'

export default defineComponent({
  name: 'ConsultationRelatedInfoPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()
    const route = useRoute()
    const consultationId = route.params.consultation_id as string

    const consultation = ref(null as Consultation | null)
    const consultationErrMessage = ref(null as string | null)

    const {
      getConsultationByConsultationIdDone,
      getConsultationByConsultationIdFunc
    } = useGetConsultationByConsultationId()

    const findConsultation = async () => {
      try {
        const response = await getConsultationByConsultationIdFunc(consultationId)
        if (!(response instanceof GetConsultationByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          consultationErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getConsultationResult()
        consultation.value = result.consultation
      } catch (e) {
        consultationErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const userRating = ref(null as UserRating | null)
    const userRatingErrMessage = ref(null as string | null)

    const {
      getUserRatingByConsultationIdDone,
      getUserRatingByConsultationIdFunc
    } = useGetUserRatingByConsultationId()

    const findUserRating = async () => {
      try {
        const response = await getUserRatingByConsultationIdFunc(consultationId)
        if (!(response instanceof GetUserRatingByConsultationIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          userRatingErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getUserRatingResult()
        userRating.value = result.user_rating
      } catch (e) {
        userRatingErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    onMounted(async () => {
      await findConsultation()
      await findUserRating()
    })

    const requestsDone = computed(() => {
      return getConsultationByConsultationIdDone.value &&
              getUserRatingByConsultationIdDone.value
    })

    return {
      requestsDone,
      consultation,
      consultationErrMessage,
      userRating,
      userRatingErrMessage
    }
  }
})
</script>
