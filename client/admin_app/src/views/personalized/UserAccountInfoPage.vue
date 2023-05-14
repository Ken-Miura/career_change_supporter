<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!requestsDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="outerErrorMessage" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <AlertMessage v-bind:message="outerErrorMessage"/>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">検索条件</h3>
          <p v-if="accountId" class="mt-4 ml-4 text-xl">アカウントID: {{ accountId }}</p>
          <p v-else-if="emailAddress" class="mt-4 ml-4 text-xl">メールアドレス: {{ emailAddress }}</p>
          <p v-else class="mt-4 ml-4 text-xl">意図しない動作です。管理者に連絡して下さい</p>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">アカウント情報</h3>
          <div class="mt-4 ml-2">
            <div v-if="userAccount">
              <div class="m-4 text-2xl grid grid-cols-3">
                <div class="mt-2 justify-self-start col-span-1">アカウントID</div><div class="mt-2 justify-self-start col-span-2">{{ userAccount.user_account_id }}</div>
                <div class="mt-2 justify-self-start col-span-1">メールアドレス</div><div class="mt-2 justify-self-start col-span-2">{{ userAccount.email_address }}</div>
                <div class="mt-2 justify-self-start col-span-1">アカウント作成日</div><div class="mt-2 justify-self-start col-span-2">{{ userAccount.created_at }}</div>
                <div class="mt-2 justify-self-start col-span-1">最終ログイン日</div><div v-if="userAccount.last_login_time" class="mt-2 justify-self-start col-span-2">{{ userAccount.last_login_time }}</div><div v-else class="mt-2 justify-self-start col-span-2">未ログイン</div>
                <div class="mt-2 justify-self-start col-span-1">無効化日時</div><div v-if="userAccount.disabled_at" class="mt-2 justify-self-start col-span-2">{{ userAccount.disabled_at }}</div><div v-else class="mt-2 justify-self-start col-span-2">無効化されていません</div>
                <div class="mt-2 justify-self-start col-span-1">二段階認証設定日</div><div v-if="userAccount.mfa_enabled_at" class="mt-2 justify-self-start col-span-2">{{ userAccount.mfa_enabled_at }}</div><div v-else class="mt-2 justify-self-start col-span-2">二段階認証は設定されていません</div>
              </div>
              <div class="mt-4 ml-2">
                <div class="text-2xl justify-self-start col-span-6 pt-3">
                  <p>アカウント無効化・有効化</p>
                </div>
                <div class="mt-2 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
                  <div class="p-4 text-xl grid grid-cols-6 justify-center items-center">
                    <div class="col-span-5">アカウントに対する操作が適正であることを確認しました</div>
                    <input v-model="accountEnableDisableConfirmation" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
                  </div>
                </div>
                <div v-if="userAccount.disabled_at">
                  <button v-on:click="enableAccount" v-bind:disabled="!accountEnableDisableConfirmation" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">有効化する</button>
                </div>
                <div v-else>
                  <button v-on:click="disableAccount" v-bind:disabled="!accountEnableDisableConfirmation" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">無効化する</button>
                </div>
                <div v-if="accountEnableDisableErrorMessage" class="mt-4">
                  <AlertMessage v-bind:message="accountEnableDisableErrorMessage"/>
                </div>
              </div>
              <div v-if="userAccount.mfa_enabled_at" class="mt-4 ml-2">
                <div class="text-2xl justify-self-start col-span-6 pt-3">
                  <p>二段階認証設定解除</p>
                </div>
                <div class="mt-2 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
                  <div class="p-4 text-xl grid grid-cols-6 justify-center items-center">
                    <div class="col-span-5">アカウントに対する操作が適正であることを確認しました</div>
                    <input v-model="disableMfaConfirmation" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
                  </div>
                </div>
                <button v-on:click="disableMfa" v-bind:disabled="!disableMfaConfirmation" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">二段階認証設定を解除する</button>
                <div v-if="disableMfaErrorMessage" class="mt-4">
                  <AlertMessage v-bind:message="disableMfaErrorMessage"/>
                </div>
              </div>
            </div>
            <div v-else>
              <p class="text-xl">アカウントが既に削除されている、または初めから存在しません。</p>
            </div>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">身分情報</h3>
          <div v-if="!identityErrMessage">
            <div v-if="identity" class="m-4 text-2xl grid grid-cols-3">
              <div class="mt-2 justify-self-start col-span-1">氏名</div><div class="mt-2 justify-self-start col-span-2">{{ identity.last_name }} {{ identity.first_name }}</div>
              <div class="mt-2 justify-self-start col-span-1">フリガナ</div><div class="mt-2 justify-self-start col-span-2">{{ identity.last_name_furigana }} {{ identity.first_name_furigana }}</div>
              <div class="mt-2 justify-self-start col-span-1">生年月日</div><div class="mt-2 justify-self-start col-span-2">{{ identity.date_of_birth.year }}年{{ identity.date_of_birth.month }}月{{ identity.date_of_birth.day }}日</div>
              <div class="mt-2 justify-self-start col-span-3">住所</div>
              <div class="mt-2 ml-3 justify-self-start col-span-1">都道府県</div><div class="mt-2 justify-self-start col-span-2">{{ identity.prefecture }}</div>
              <div class="mt-2 ml-3 justify-self-start col-span-1">市区町村</div><div class="mt-2 justify-self-start col-span-2">{{ identity.city }}</div>
              <div class="mt-2 ml-3 justify-self-start col-span-1">番地</div><div class="mt-2 justify-self-start col-span-2">{{ identity.address_line1 }}</div>
              <div v-if="identity.address_line2 !== null" class="mt-2 ml-3 justify-self-start col-span-1">建物名・部屋番号</div><div v-if="identity.address_line2 !== null" class="mt-2 justify-self-start col-span-2">{{ identity.address_line2 }}</div>
              <div class="mt-2 justify-self-start col-span-1">電話番号</div><div class="mt-2 justify-self-start col-span-2">{{ identity.telephone_number }}</div>
            </div>
            <div v-else class="m-4 text-2xl">
              身分情報は見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="identityErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">職務経歴</h3>
          <div v-if="!careersErrMessage">
            <div v-if="careers.length !== 0" class="m-4 text-2xl grid grid-cols-3">
              {{ careers }}
            </div>
            <div v-else class="m-4 text-2xl">
              職務経歴は見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="careersErrMessage"/>
          </div>
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
import { useStore } from 'vuex'
import { UserAccountSearchParam } from '@/util/personalized/user-account-search/UserAccountSearchParam'
import { Message } from '@/util/Message'
import { usePostUserAccountRetrieval } from '@/util/personalized/user-account-info/usePostUserAccountRetrieval'
import { ApiErrorResp } from '@/util/ApiError'
import { UserAccountRetrievalResp } from '@/util/personalized/user-account-info/UserAccountRetrievalResp'
import { Code, createErrorMessage } from '@/util/Error'
import { useRouter } from 'vue-router'
import { UserAccount } from '@/util/personalized/user-account-info/UserAccount'
import { Identity } from '@/util/personalized/Identity'
import { useGetIdentityOptionByUserAccountId } from '@/util/personalized/user-account-info/useGetIdentityOptionByUserAccountId'
import { GetIdentityOptionByUserAccountIdResp } from '@/util/personalized/user-account-info/GetIdentityOptionByUserAccountIdResp'
import { useGetCareersByUserAccountId } from '@/util/personalized/user-account-info/useGetCareersByUserAccountId'
import { CareersWithId } from '@/util/personalized/user-account-info/CareersWithId'
import { GetCareersByUserAccountIdResp } from '@/util/personalized/user-account-info/GetCareersByUserAccountIdResp'

export default defineComponent({
  name: 'UserAccountInfoPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()
    const store = useStore()

    const accountId = ref(null as number | null)
    const emailAddress = ref(null as string | null)
    const outerErrorMessage = ref(null as string | null)

    const userAccount = ref(null as UserAccount | null)
    const {
      postUserAccountRetrievalDone,
      postUserAccountRetrievalByUserAccountIdFunc,
      postUserAccountRetrievalByEmailAddressFunc
    } = usePostUserAccountRetrieval()

    const getUserAccountByEitherAccountIdOrEmailAddress = async (accountId: number | null, emailAddress: string | null): Promise<UserAccountRetrievalResp | ApiErrorResp> => {
      if (accountId) {
        return postUserAccountRetrievalByUserAccountIdFunc(accountId)
      } else if (emailAddress) {
        return postUserAccountRetrievalByEmailAddressFunc(emailAddress)
      } else {
        throw new Error('Both accountId and emailAddress are null')
      }
    }

    const getUserAccount = async (accountId: number | null, emailAddress: string | null) => {
      const response = await getUserAccountByEitherAccountIdOrEmailAddress(accountId, emailAddress)
      if (!(response instanceof UserAccountRetrievalResp)) {
        if (!(response instanceof ApiErrorResp)) {
          throw new Error(`unexpected result on getting request detail: ${response}`)
        }
        const code = response.getApiError().getCode()
        if (code === Code.UNAUTHORIZED) {
          await router.push('/login')
          return
        }
        outerErrorMessage.value = createErrorMessage(response.getApiError().getCode())
        return
      }
      const result = response.getResult()
      userAccount.value = result.user_account
    }

    const accountEnableDisableConfirmation = ref(false)
    const accountEnableDisableErrorMessage = ref(null as string | null)
    const disableAccount = async () => {
      console.log('disableAccount') // 更新後のUserAccountを返してもらうようにする
    }
    const enableAccount = async () => {
      console.log('enableAccount') // 更新後のUserAccountを返してもらうようにする
    }

    const disableMfaConfirmation = ref(false)
    const disableMfaErrorMessage = ref(null as string | null)
    const disableMfa = async () => {
      console.log('disableMfa') // 更新後のUserAccountを返してもらうようにする
    }

    const selectUserAccountId = (userAccount: UserAccount | null, userAccountId: number | null) => {
      if (userAccount) {
        return userAccount.user_account_id
      }
      if (userAccountId) {
        return userAccountId
      }
      return null
    }

    const identity = ref(null as Identity | null)
    const {
      getIdentityOptionByUserAccountIdDone,
      getIdentityOptionByUserAccountIdFunc
    } = useGetIdentityOptionByUserAccountId()
    const identityErrMessage = ref(null as string | null)

    const findIdentity = async (accountId: number) => {
      const response = await getIdentityOptionByUserAccountIdFunc(accountId.toString())
      if (!(response instanceof GetIdentityOptionByUserAccountIdResp)) {
        if (!(response instanceof ApiErrorResp)) {
          throw new Error(`unexpected result on getting request detail: ${response}`)
        }
        const code = response.getApiError().getCode()
        if (code === Code.UNAUTHORIZED) {
          await router.push('/login')
          return
        }
        identityErrMessage.value = createErrorMessage(response.getApiError().getCode())
        return
      }
      const result = response.getIdentityResult()
      identity.value = result.identity_option
    }

    const careers = ref([] as CareersWithId[])
    const {
      getCareersByUserAccountIdDone,
      getCareersByUserAccountIdFunc
    } = useGetCareersByUserAccountId()
    const careersErrMessage = ref(null as string | null)

    const findCareers = async (accountId: number) => {
      const response = await getCareersByUserAccountIdFunc(accountId.toString())
      if (!(response instanceof GetCareersByUserAccountIdResp)) {
        if (!(response instanceof ApiErrorResp)) {
          throw new Error(`unexpected result on getting request detail: ${response}`)
        }
        const code = response.getApiError().getCode()
        if (code === Code.UNAUTHORIZED) {
          await router.push('/login')
          return
        }
        careersErrMessage.value = createErrorMessage(response.getApiError().getCode())
        return
      }
      const result = response.getCareersResult()
      careers.value = result.careers
    }

    onMounted(async () => {
      const param = store.state.userAccountSearchParam as UserAccountSearchParam
      if (!param) {
        outerErrorMessage.value = Message.USER_ACCOUNT_SEARCH_PARAM_IS_NULL
        return
      }
      if (param.accountId === null && param.emailAddress === null) {
        outerErrorMessage.value = Message.BOTH_ACCOUNT_ID_AND_EMAIL_ADDRESS_ARE_EMPTY_MESSAGE
        return
      }
      if (param.accountId !== null && param.emailAddress !== null) {
        outerErrorMessage.value = Message.BOTH_ACCOUNT_ID_AND_EMAIL_ADDRESS_ARE_FILLED_MESSAGE
        return
      }
      accountId.value = param.accountId
      emailAddress.value = param.emailAddress

      await getUserAccount(param.accountId, param.emailAddress)

      const accId = selectUserAccountId(userAccount.value, param.accountId)
      if (!accId) {
        return
      }

      await findIdentity(accId)
      await findCareers(accId)
    })

    const requestsDone = computed(() => {
      return (postUserAccountRetrievalDone.value &&
        getIdentityOptionByUserAccountIdDone.value &&
        getCareersByUserAccountIdDone.value)
    })

    return {
      requestsDone,
      accountId,
      emailAddress,
      userAccount,
      accountEnableDisableConfirmation,
      accountEnableDisableErrorMessage,
      disableAccount,
      enableAccount,
      disableMfaConfirmation,
      disableMfaErrorMessage,
      disableMfa,
      identity,
      identityErrMessage,
      careers,
      careersErrMessage,
      outerErrorMessage
    }
  }
})
</script>
