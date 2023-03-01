<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getUnratedItemsDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="errMessage">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="errMessage"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 data-test="unrated-consultants-label" class="font-bold text-2xl">相談を行ったコンサルタント</h3>
          <p data-test="unrated-consultants-description" class="mt-2 ml-2 text-xl">相談日時が古い方から最大{{ MAX_NUM_OF_UNRATED_CONSULTANTS }}件分表示されます。{{ MAX_NUM_OF_UNRATED_CONSULTANTS }}件を超えた分は表示されているコンサルタントの評価を終えると表示されます。</p>
          <div v-if="unratedItemsResult.unrated_consultants.length !== 0" class="m-2 text-2xl">
            <ul>
              <li v-for="unrated_consultant in unratedItemsResult.unrated_consultants" v-bind:key="unrated_consultant.consultant_rating_id">
                <div v-bind:data-test="'consultant-rating-id-' + unrated_consultant.consultant_rating_id" class="mt-4">
                  <div data-test="consultant-id-label" class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">コンサルタントID（{{ unrated_consultant.consultant_id }}）</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div data-test="consultation-date-time" class="mt-4 justify-self-start col-span-2">相談日時：{{ unrated_consultant.meeting_date_time_in_jst.year }}年{{ unrated_consultant.meeting_date_time_in_jst.month }}月{{ unrated_consultant.meeting_date_time_in_jst.day }}日{{ unrated_consultant.meeting_date_time_in_jst.hour }}時</div>
                    <button data-test="move-to-rate-consultant-page" v-on:click="moveToRateConsultantPage(unrated_consultant.consultant_rating_id, unrated_consultant.consultant_id, unrated_consultant.meeting_date_time_in_jst)" class="mt-2 col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">評価する</button>
                  </div>
                </div>
              </li>
            </ul>
          </div>
          <div v-else class="m-6 text-2xl">
            <p data-test="no-unrated-consultants-label" class="text-xl">未評価のコンサルタントはいません</p>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 data-test="unrated-users-label" class="font-bold text-2xl">相談を受け付けたユーザー</h3>
          <p data-test="unrated-users-description" class="mt-2 ml-2 text-xl">相談日時が古い方から最大{{ MAX_NUM_OF_UNRATED_USERS }}件分表示されます。{{ MAX_NUM_OF_UNRATED_USERS }}件を超えた分は表示されているユーザーの評価を終えると表示されます。</p>
          <div v-if="unratedItemsResult.unrated_users.length !== 0" class="m-4 text-2xl">
            <ul>
              <li v-for="unrated_user in unratedItemsResult.unrated_users" v-bind:key="unrated_user.user_rating_id">
                <div v-bind:data-test="'user-rating-id-' + unrated_user.user_rating_id" class="mt-4">
                  <div data-test="user-account-id-label" class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">ユーザーID（{{ unrated_user.user_account_id }}）からの相談</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div data-test="consultation-date-time" class="mt-4 justify-self-start col-span-2">相談日時：{{ unrated_user.meeting_date_time_in_jst.year }}年{{ unrated_user.meeting_date_time_in_jst.month }}月{{ unrated_user.meeting_date_time_in_jst.day }}日{{ unrated_user.meeting_date_time_in_jst.hour }}時</div>
                    <button data-test="move-to-rate-user-page" v-on:click="moveToRateUserPage(unrated_user.user_rating_id, unrated_user.user_account_id, unrated_user.meeting_date_time_in_jst)" class="mt-2 col-span-1 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">評価する</button>
                  </div>
                </div>
              </li>
            </ul>
          </div>
          <div v-else class="m-6 text-2xl">
            <p data-test="no-unrated-users-label" class="text-xl">未評価のユーザーはいません</p>
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
import { defineComponent, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useGetUnratedItems } from '@/util/personalized/unrated-item-list/useGetUnratedItems'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { UnratedItemsResultResp } from '@/util/personalized/unrated-item-list/UnratedItemsResultResp'
import { Message } from '@/util/Message'
import { UnratedItemsResult } from '@/util/personalized/unrated-item-list/UnratedItemsResult'
import { MAX_NUM_OF_UNRATED_CONSULTANTS, MAX_NUM_OF_UNRATED_USERS } from '@/util/personalized/unrated-item-list/MaxNumOfUnratedItems'
import { ConsultationDateTime } from '@/util/personalized/ConsultationDateTime'

export default defineComponent({
  name: 'UnratedItemListPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const errMessage = ref(null as string | null)
    const router = useRouter()
    const {
      getUnratedItemsDone,
      getUnratedItemsFunc
    } = useGetUnratedItems()
    const unratedItemsResult = ref({ unrated_consultants: [], unrated_users: [] } as UnratedItemsResult)

    onMounted(async () => {
      try {
        const resp = await getUnratedItemsFunc()
        if (!(resp instanceof UnratedItemsResultResp)) {
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
        unratedItemsResult.value = resp.getUnratedItemsResult()
      } catch (e) {
        errMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const moveToRateConsultantPage = async (consultantRatingId: number, consultantId: number, consultationDateTime: ConsultationDateTime) => {
      await router.push(`/rate-consultant/${consultantRatingId}?consultant-id=${consultantId}&year=${consultationDateTime.year}&month=${consultationDateTime.month}&day=${consultationDateTime.day}&hour=${consultationDateTime.hour}`)
    }

    const moveToRateUserPage = async (userRatingId: number, userAccountId: number, consultationDateTime: ConsultationDateTime) => {
      await router.push(`/rate-user/${userRatingId}?user-id=${userAccountId}&year=${consultationDateTime.year}&month=${consultationDateTime.month}&day=${consultationDateTime.day}&hour=${consultationDateTime.hour}`)
    }

    return {
      getUnratedItemsDone,
      errMessage,
      unratedItemsResult,
      moveToRateConsultantPage,
      moveToRateUserPage,
      MAX_NUM_OF_UNRATED_CONSULTANTS,
      MAX_NUM_OF_UNRATED_USERS
    }
  }
})
</script>
