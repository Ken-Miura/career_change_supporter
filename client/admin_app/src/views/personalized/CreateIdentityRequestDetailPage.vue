<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="waitingRequestDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="error.exists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="error.message"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">本人確認依頼（新規）詳細</h3>
          <div v-if="detail !== null">
            <div class="m-4 text-2xl grid grid-cols-3">
              <div class="mt-2 justify-self-start col-span-1">名前</div><div class="justify-self-start col-span-2">{{ detail.last_name }} {{ detail.first_name }}</div>
              <div class="mt-2 justify-self-start col-span-1">フリガナ</div><div class="justify-self-start col-span-2">{{ detail.last_name_furigana }} {{ detail.first_name_furigana }}</div>
              <div class="mt-2 justify-self-start col-span-1">生年月日</div><div class="justify-self-start col-span-2">{{ detail.date_of_birth.year }}年{{ detail.date_of_birth.month }}月{{ detail.date_of_birth.day }}日</div>
              <div class="mt-2 justify-self-start col-span-3">住所</div>
              <div class="mt-2 ml-3 justify-self-start col-span-1">都道府県</div><div class="justify-self-start col-span-2">{{ detail.prefecture }}</div>
              <div class="mt-2 ml-3 justify-self-start col-span-1">市区町村</div><div class="justify-self-start col-span-2">{{ detail.city }}</div>
              <div class="mt-2 ml-3 justify-self-start col-span-1">番地</div><div class="justify-self-start col-span-2">{{ detail.address_line1 }}</div>
              <div v-if="detail.address_line2 !== null" class="mt-2 ml-3 justify-self-start col-span-1">建物名・部屋番号</div><div v-if="detail.address_line2 !== null" class="justify-self-start col-span-2">{{ detail.address_line2 }}</div>
              <div class="mt-2 justify-self-start col-span-1">電話番号</div><div class="justify-self-start col-span-2">{{ detail.telephone_number }}</div>
            </div>
            <div class="m-2 text-2xl">
              <div class="mt-2">身分証明書画像（表面）</div>
              <img class="mt-2" v-bind:src="image1Url" />
            </div>
            <div v-if="image2Url !== null" class="m-2 text-2xl">
              <div class="mt-2">身分証明書画像（裏面）</div>
              <img class="mt-2" v-bind:src="image2Url" />
            </div>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">生年月日が同じユーザー</h3>
          <p class="mt-2 text-lg">既に登録されているユーザーが新規に本人確認依頼をしてきていないか確認して下さい。</p>
          <div v-if="users.length === 0">
            <div class="m-4 text-xl">
              <div class="mt-2">生年月日が同じユーザーはいません。</div>
            </div>
          </div>
          <div v-else>
            <ul>
              <li v-for="user in users" v-bind:key="user">
                <div class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">ユーザーアカウントID: {{ user.user_account_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div class="mt-2 justify-self-start col-span-1">名前</div><div class="justify-self-start col-span-2">{{ user.last_name }} {{ user.first_name }}</div>
                    <div class="mt-2 justify-self-start col-span-1">フリガナ</div><div class="justify-self-start col-span-2">{{ user.last_name_furigana }} {{ user.first_name_furigana }}</div>
                    <div class="mt-2 justify-self-start col-span-1">生年月日</div><div class="justify-self-start col-span-2">{{ user.date_of_birth.year }}年{{ user.date_of_birth.month }}月{{ user.date_of_birth.day }}日</div>
                    <div class="mt-2 justify-self-start col-span-3">住所</div>
                    <div class="mt-2 ml-3 justify-self-start col-span-1">都道府県</div><div class="justify-self-start col-span-2">{{ user.prefecture }}</div>
                    <div class="mt-2 ml-3 justify-self-start col-span-1">市区町村</div><div class="justify-self-start col-span-2">{{ user.city }}</div>
                    <div class="mt-2 ml-3 justify-self-start col-span-1">番地</div><div class="justify-self-start col-span-2">{{ user.address_line1 }}</div>
                    <div v-if="user.address_line2 !== null" class="mt-2 ml-3 justify-self-start col-span-1">建物名・部屋番号</div><div v-if="user.address_line2 !== null" class="justify-self-start col-span-2">{{ user.address_line2 }}</div>
                    <div class="mt-2 justify-self-start col-span-1">電話番号</div><div class="justify-self-start col-span-2">{{ user.telephone_number }}</div>
                  </div>
                </div>
              </li>
            </ul>
          </div>
        </div>
        <div class="flex flex-row justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <button v-on:click="approveReq" class="w-1/2 bg-gray-600 hover:bg-gray-700 text-white font-bold mx-2 px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">承認する</button>
          <button v-on:click="chooseRejectionReason" class="w-1/2 bg-gray-600 hover:bg-gray-700 text-white font-bold mx-2 px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">拒否理由を選ぶ</button>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { User } from '@/util/personalized/create-identity-request-detail/User'
import { CreateIdentityRequestDetail } from '@/util/personalized/create-identity-request-detail/CreateIdentityRequestDetail'
import { useGetCreateIdentityRequestDetail } from '@/util/personalized/create-identity-request-detail/useGetCreateIdentityRequestDetail'
import { GetCreateIdentityRequestDetailResp } from '@/util/personalized/create-identity-request-detail/GetCreateIdentityRequestDetailResp'
import { useGetUsersByDateOfBirth } from '@/util/personalized/create-identity-request-detail/useGetUsersByDateOfBirth'
import { usePostCreateIdentityRequestApproval } from '@/util/personalized/create-identity-request-detail/usePostCreateIdentityRequestApproval'
import { Code, createErrorMessage } from '@/util/Error'
import { ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'
import { GetUsersByDateOfBirthResp } from '@/util/personalized/create-identity-request-detail/GetUsersByDateOfBirthResp'
import { PostCreateIdentityRequestApprovalResp } from '@/util/personalized/create-identity-request-detail/PostCreateIdentityRequestApprovalResp'

export default defineComponent({
  name: 'CreateIdentityRequestDetailPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const error = reactive({
      exists: false,
      message: ''
    })
    const detail = ref(null as CreateIdentityRequestDetail | null)
    const users = ref([] as User[])
    const route = useRoute()
    const router = useRouter()
    const userAccountId = route.params.account_id as string
    const image1Url = computed(() => {
      if (detail.value === null) {
        return ''
      }
      return `/admin/api/identity-images/${userAccountId}/${detail.value.image1_file_name_without_ext}`
    })
    const image2Url = computed(() => {
      if (detail.value === null) {
        return null
      }
      const image2Name = detail.value.image2_file_name_without_ext
      if (image2Name === null) {
        return null
      }
      return `/admin/api/identity-images/${userAccountId}/${image2Name}`
    })
    const {
      waitingGetCreateIdentityRequestDetailDone,
      getCreateIdentityRequestDetailFunc
    } = useGetCreateIdentityRequestDetail()
    const {
      waitingGetUsersByDateOfBirthDone,
      getUsersByDateOfBirthFunc
    } = useGetUsersByDateOfBirth()
    const {
      waitingPostCreateIdentityRequestApprovalDone,
      postCreateIdentityRequestApprovalFunc
    } = usePostCreateIdentityRequestApproval()
    const waitingRequestDone = computed(() => {
      return waitingGetCreateIdentityRequestDetailDone.value || waitingGetUsersByDateOfBirthDone.value || waitingPostCreateIdentityRequestApprovalDone.value
    })
    const getUsers = async (year: number, month: number, day: number) => {
      const response = await getUsersByDateOfBirthFunc(year, month, day)
      if (!(response instanceof GetUsersByDateOfBirthResp)) {
        if (!(response instanceof ApiErrorResp)) {
          throw new Error(`unexpected result on getting users: ${response}`)
        }
        const code = response.getApiError().getCode()
        if (code === Code.UNAUTHORIZED) {
          await router.push('/login')
          return
        }
        error.exists = true
        error.message = createErrorMessage(response.getApiError().getCode())
        return
      }
      users.value = response.getUsers()
    }
    onMounted(async () => {
      try {
        const response = await getCreateIdentityRequestDetailFunc(userAccountId)
        if (!(response instanceof GetCreateIdentityRequestDetailResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          error.exists = true
          error.message = createErrorMessage(response.getApiError().getCode())
          return
        }
        detail.value = response.getDetail()
        const dateOfBirth = detail.value.date_of_birth
        await getUsers(dateOfBirth.year, dateOfBirth.month, dateOfBirth.day)
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const approveReq = async () => {
      try {
        const response = await postCreateIdentityRequestApprovalFunc(parseInt(userAccountId))
        if (!(response instanceof PostCreateIdentityRequestApprovalResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          error.exists = true
          error.message = createErrorMessage(response.getApiError().getCode())
          return
        }
        await router.push('/create-identity-request-approval')
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const chooseRejectionReason = async () => {
      await router.push({ name: 'CreateIdentityRequestRejectionDetailPage', params: { account_id: userAccountId } })
    }

    return { error, detail, image1Url, image2Url, users, waitingRequestDone, approveReq, chooseRejectionReason }
  }
})
</script>
