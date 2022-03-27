<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getProfileDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="errorExists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="errorMessage"/>
        </div>
      </div>
      <div v-else>
        <div data-test="email-address" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">Eメールアドレス</h3>
          <p class="mt-2 text-lg">登録したEメールアドレスです。他のユーザーに公開されることはありません。</p>
          <p class="mt-4 ml-4 text-2xl">{{ emailAddress }}</p>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">ユーザー情報</h3>
          <p class="mt-2 text-lg">本人確認のために利用される情報です（本人確認の完了後、相談申し込みが可能となります）ユーザー情報が他のユーザーに公開されることはありません。</p>
          <div v-if="identity !== null" data-test="identity-set" class="m-4 text-2xl grid grid-cols-3">
            <div class="mt-2 justify-self-start col-span-1">名前</div><div class="justify-self-start col-span-2">{{ identity.last_name }} {{ identity.first_name }}</div>
            <div class="mt-2 justify-self-start col-span-1">フリガナ</div><div class="justify-self-start col-span-2">{{ identity.last_name_furigana }} {{ identity.first_name_furigana }}</div>
            <div class="mt-2 justify-self-start col-span-1">生年月日</div><div class="justify-self-start col-span-2">{{ identity.date_of_birth.year }}年{{ identity.date_of_birth.month }}月{{ identity.date_of_birth.day }}日</div>
            <div class="mt-2 justify-self-start col-span-3">住所</div>
            <div class="mt-2 ml-3 justify-self-start col-span-1">都道府県</div><div class="justify-self-start col-span-2">{{ identity.prefecture }}</div>
            <div class="mt-2 ml-3 justify-self-start col-span-1">市区町村</div><div class="justify-self-start col-span-2">{{ identity.city }}</div>
            <div class="mt-2 ml-3 justify-self-start col-span-1">番地</div><div class="justify-self-start col-span-2">{{ identity.address_line1 }}</div>
            <div v-if="identity.address_line2 !== null" class="mt-2 ml-3 justify-self-start col-span-1">建物名・部屋番号</div><div v-if="identity.address_line2 !== null" class="justify-self-start col-span-2">{{ identity.address_line2 }}</div>
            <div class="mt-2 justify-self-start col-span-1">電話番号</div><div class="justify-self-start col-span-2">{{ identity.telephone_number }}</div>
          </div>
          <p v-else data-test="no-identity-set" class="m-4 text-xl">ユーザー情報が設定されていません。</p>
          <button data-test="move-to-identity-page-button" v-on:click="moveToIdentityPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">ユーザー情報を編集する</button>
        </div>
        <div data-test="career" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">職務経歴</h3>
          <p class="mt-2 text-lg">相談受け付けを行うために必要となる情報です。<span class=" text-red-500">相談申し込みの判断に使われるため、他のユーザーに公開されます。</span>入社日と退社日は在籍年数（3年未満、3年以上5年未満、5年以上10年未満、10年以上15年未満、15年以上20年未満、20年以上）という形に変換され、そのまま公開されることはありません。</p>
          <div v-if="careers.length === 0" data-test="no-careers-set" class="mt-4 ml-4 text-xl">職務経歴は登録されていません。</div>
          <div v-else data-test="careers-set">
            <ul>
              <li v-for="(career, index) in careers" v-bind:key="career">
                <div class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">職務経歴{{ index + 1 }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div class="mt-2 justify-self-start col-span-1">勤務先名称</div><div class="justify-self-start col-span-2">{{ career.company_name }}</div>
                    <div class="mt-2 justify-self-start col-span-1">雇用形態</div>
                    <div class="justify-self-start col-span-2">
                      <div v-if="career.contract_type === 'regular'">
                        正社員
                      </div>
                      <div v-else-if="career.contract_type === 'contract'">
                        契約社員
                      </div>
                      <div v-else-if="career.contract_type === 'other'">
                        その他
                      </div>
                      <div v-else>
                        その他
                      </div>
                    </div>
                    <div class="mt-2 justify-self-start col-span-1">入社日</div><div class="justify-self-start col-span-2">{{ career.career_start_date.year }}年{{ career.career_start_date.month }}月{{ career.career_start_date.day }}日</div>
                    <div v-if="career.career_end_date !== null" class="mt-2 justify-self-start col-span-1">退社日</div><div v-if="career.career_end_date !== null" class="justify-self-start col-span-2">{{ career.career_end_date.year }}年{{ career.career_end_date.month }}月{{ career.career_end_date.day }}日</div>
                    <button data-test="move-to-edit-career-page-button" v-on:click="moveToEditCareerPage(career.career_id)" class="mt-4 col-span-3 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">詳細を確認・編集する</button>
                  </div>
                </div>
              </li>
            </ul>
          </div>
          <button data-test="move-to-add-career-page-button" v-on:click="moveToAddCareerPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">職務経歴を追加する</button>
          <AlertMessage v-bind:class="['mt-6', { 'hidden': canAddCareer }]" v-bind:message="canAddCareerErrMessage"/>
        </div>
        <div data-test="fee-per-hour-in-yen" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">相談一回（１時間）の相談料</h3>
          <p class="mt-2 text-lg">相談受け付けを行うために必要となる情報です。<span class=" text-red-500">相談申し込みの判断に使われるため、他のユーザーに公開されます。</span>相談料から本サイト利用の手数料（30パーセント）が差し引かれた金額が報酬として計上されます。</p>
          <div v-if="feePerHourInYen !== null" data-test="fee-per-hour-in-yen-set" class="flex justify-end">
            <p class="m-4 text-2xl">{{ feePerHourInYen }}円</p>
          </div>
          <p v-else data-test="no-fee-per-hour-in-yen-set" class="m-4 text-xl">相談料が設定されていません。</p>
          <button data-test="move-to-fee-per-hour-in-yen-page-button" v-on:click="moveToFeePerHourInYenPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">相談料を編集する</button>
          <AlertMessage v-bind:class="['mt-6', { 'hidden': canEditFeePerHourInYen }]" v-bind:message="canEditFeePerHourInYenErrMessage"/>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <button data-test="move-to-delete-account-confirmation-page-button" v-on:click="moveToDeleteAccountConfirmationPage" class="bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">アカウントを削除する</button>
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
import { GetProfileResp } from '@/util/personalized/profile/GetProfileResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Identity } from '@/util/personalized/profile/Identity'
import { useGetProfile } from '@/util/personalized/profile/useGetProfile'
import { Career } from '@/util/personalized/profile/Career'
import { Message } from '@/util/Message'
import { Code, createErrorMessage } from '@/util/Error'
import { useStore } from 'vuex'
import { SET_CAREERS, SET_FEE_PER_HOUR_IN_YEN, SET_IDENTITY } from '@/store/mutationTypes'

export default defineComponent({
  name: 'ProfilePage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const { getProfileDone, getProfileFunc } = useGetProfile()
    const emailAddress = ref('')
    const identity = ref(null as Identity | null)
    const careers = ref([] as Career[])
    const canAddCareer = ref(true)
    const canAddCareerErrMessage = ref('')
    const feePerHourInYen = ref(0 as number | null)
    const canEditFeePerHourInYen = ref(true)
    const canEditFeePerHourInYenErrMessage = ref('')
    const router = useRouter()
    const store = useStore()
    const errorExists = ref(false)
    const errorMessage = ref('')
    onMounted(async () => {
      try {
        const response = await getProfileFunc()
        if (response instanceof GetProfileResp) {
          const profile = response.getProfile()
          /* eslint-disable camelcase */
          emailAddress.value = profile.email_address
          identity.value = profile.identity
          careers.value = profile.careers
          feePerHourInYen.value = profile.fee_per_hour_in_yen
          /* eslint-enable camelcase */
          store.commit(SET_IDENTITY, profile.identity)
          store.commit(SET_CAREERS, profile.careers)
          store.commit(SET_FEE_PER_HOUR_IN_YEN, profile.fee_per_hour_in_yen)
        } else if (response instanceof ApiErrorResp) {
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          }
          errorExists.value = true
          errorMessage.value = createErrorMessage(response.getApiError().getCode())
        } else {
          throw new Error(`unexpected result: ${response}`)
        }
      } catch (e) {
        errorExists.value = true
        errorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })
    const moveToIdentityPage = async () => {
      await router.push('/identity')
    }
    const moveToAddCareerPage = async () => {
      const identity = store.state.identity
      if (identity === null) {
        canAddCareer.value = false
        canAddCareerErrMessage.value = Message.NO_IDENTITY_FOUND
        return
      }
      await router.push('/careers')
    }
    const moveToEditCareerPage = async (careerId: number) => {
      await router.push({ name: 'EditCareerPage', params: { career_id: careerId } })
    }
    const moveToFeePerHourInYenPage = async () => {
      const identity = store.state.identity
      if (identity === null) {
        canEditFeePerHourInYen.value = false
        canEditFeePerHourInYenErrMessage.value = Message.NO_IDENTITY_FOUND
        return
      }
      await router.push('/fee-per-hour-in-yen')
    }
    const moveToDeleteAccountConfirmationPage = async () => {
      await router.push('/delete-account-confirmation')
    }
    return {
      getProfileDone,
      emailAddress,
      identity,
      careers,
      canAddCareer,
      canAddCareerErrMessage,
      feePerHourInYen,
      canEditFeePerHourInYen,
      canEditFeePerHourInYenErrMessage,
      errorExists,
      errorMessage,
      moveToIdentityPage,
      moveToAddCareerPage,
      moveToEditCareerPage,
      moveToFeePerHourInYenPage,
      moveToDeleteAccountConfirmationPage
    }
  }
})
</script>
