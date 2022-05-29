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
        <div data-test="req-detail" class="flex flex-col justify-center bg-white max-w-6xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">本人確認依頼（更新）詳細</h3>
          <p class="m-4 text-xl"><span class="font-bold">更新後の表示</span>が身分証明書画像と一致するか確認して下さい（身分証明書画像が運転免許証の場合、更新前の住所も一致しているか確認して下さい）</p>
          <div v-if="detail !== null && identity !== null">
            <div class="m-4 text-2xl grid grid-cols-8">
              <div class="mt-2 justify-self-start col-span-2"></div><div class="mt-2 justify-self-start font-bold col-span-3">更新前</div><div class="mt-2 justify-self-start font-bold col-span-3">更新後</div>
              <div class="mt-2 justify-self-start col-span-2">氏名</div><div class="mt-2 justify-self-start col-span-3">{{ identity.last_name }} {{ identity.first_name }}</div><div class="mt-2 justify-self-start col-span-3">{{ detail.last_name }} {{ detail.first_name }}</div>
              <div class="mt-2 justify-self-start col-span-2">フリガナ</div><div class="mt-2 justify-self-start col-span-3">{{ identity.last_name_furigana }} {{ identity.first_name_furigana }}</div><div class="mt-2 justify-self-start col-span-3">{{ detail.last_name_furigana }} {{ detail.first_name_furigana }}</div>
              <div class="mt-2 justify-self-start col-span-2">生年月日</div><div class="mt-2 justify-self-start col-span-3">{{ identity.date_of_birth.year }}年{{ identity.date_of_birth.month }}月{{ identity.date_of_birth.day }}日</div><div class="mt-2 justify-self-start col-span-3">{{ detail.date_of_birth.year }}年{{ detail.date_of_birth.month }}月{{ detail.date_of_birth.day }}日</div>
              <div class="mt-2 justify-self-start col-span-8">住所</div>
              <div class="mt-2 ml-3 justify-self-start col-span-2">都道府県</div><div class="mt-2 justify-self-start col-span-3">{{ identity.prefecture }}</div><div class="mt-2 justify-self-start col-span-3">{{ detail.prefecture }}</div>
              <div class="mt-2 ml-3 justify-self-start col-span-2">市区町村</div><div class="mt-2 justify-self-start col-span-3">{{ identity.city }}</div><div class="mt-2 justify-self-start col-span-3">{{ detail.city }}</div>
              <div class="mt-2 ml-3 justify-self-start col-span-2">番地</div><div class="mt-2 justify-self-start col-span-3">{{ identity.address_line1 }}</div><div class="mt-2 justify-self-start col-span-3">{{ detail.address_line1 }}</div>
              <div v-if="identity.address_line2 !== null || detail.address_line2 !== null" class="col-span-8 grid grid-cols-8">
                <div class="mt-2 ml-3 justify-self-start col-span-2">建物名・部屋番号</div>
                <div v-if="identity.address_line2 !== null" class="mt-2 justify-self-start col-span-3">{{ identity.address_line2 }}</div><div v-else class="mt-2 justify-self-start col-span-3"></div>
                <div v-if="detail.address_line2 !== null" class="mt-2 justify-self-start col-span-3">{{ detail.address_line2 }}</div><div v-else class="mt-2 justify-self-start col-span-3"></div>
              </div>
              <div class="mt-2 justify-self-start col-span-2">電話番号</div><div class="mt-2 justify-self-start col-span-3">{{ identity.telephone_number }}</div><div class="mt-2 justify-self-start col-span-3">{{ detail.telephone_number }}</div>
            </div>
            <div class="m-2 text-2xl">
              <div class="mt-2">身分証明書画像（表面）</div>
              <img data-test="req-detail-image1" class="mt-2" v-bind:src="image1Url" />
            </div>
            <div v-if="image2Url !== null" class="m-2 text-2xl">
              <div class="mt-2">身分証明書画像（裏面）</div>
              <img data-test="req-detail-image2" class="mt-2" v-bind:src="image2Url" />
            </div>
          </div>
        </div>
        <div class="flex flex-row justify-center bg-white max-w-6xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <button data-test="approve-req-button" v-on:click="approveReq" class="w-1/2 bg-gray-600 hover:bg-gray-700 text-white font-bold mx-2 px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">承認する</button>
          <button data-test="choose-rejection-reason-button" v-on:click="chooseRejectionReason" class="w-1/2 bg-gray-600 hover:bg-gray-700 text-white font-bold mx-2 px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">拒否理由を選ぶ</button>
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
import { UpdateIdentityRequestDetail } from '@/util/personalized/update-identity-request-detail/UpdateIdentityRequestDetail'
import { useGetUpdateIdentityRequestDetail } from '@/util/personalized/update-identity-request-detail/useGetUpdateIdentityRequestDetail'
import { GetUpdateIdentityRequestDetailResp } from '@/util/personalized/update-identity-request-detail/GetUpdateIdentityRequestDetailResp'
import { useGetIdentityByUserAccountId } from '@/util/personalized/useGetIdentityByUserAccountId'
import { usePostUpdateIdentityRequestApproval } from '@/util/personalized/update-identity-request-detail/usePostUpdateIdentityRequestApproval'
import { Code, createErrorMessage } from '@/util/Error'
import { ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'
import { Identity } from '@/util/personalized/Identity'
import { GetIdentityByUserAccountIdResp } from '@/util/personalized/GetIdentityByUserAccountIdResp'
import { PostUpdateIdentityRequestApprovalResp } from '@/util/personalized/update-identity-request-detail/PostUpdateIdentityRequestApprovalResp'

export default defineComponent({
  name: 'UpdateIdentityRequestDetailPage',
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
    const detail = ref(null as UpdateIdentityRequestDetail | null)
    const identity = ref(null as Identity | null)
    const route = useRoute()
    const router = useRouter()
    const userAccountId = route.params.user_account_id as string
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
      waitingGetUpdateIdentityRequestDetailDone,
      getUpdateIdentityRequestDetailFunc
    } = useGetUpdateIdentityRequestDetail()
    const {
      waitingGetIdentityByUserAccountIdDone,
      getIdentityByUserAccountIdFunc
    } = useGetIdentityByUserAccountId()
    const {
      waitingPostUpdateIdentityRequestApprovalDone,
      postUpdateIdentityRequestApprovalFunc
    } = usePostUpdateIdentityRequestApproval()
    const waitingRequestDone = computed(() => {
      return waitingGetUpdateIdentityRequestDetailDone.value || waitingGetIdentityByUserAccountIdDone.value || waitingPostUpdateIdentityRequestApprovalDone.value
    })
    const getIdentity = async (userAccoundId: string) => {
      const response = await getIdentityByUserAccountIdFunc(userAccoundId)
      if (!(response instanceof GetIdentityByUserAccountIdResp)) {
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
      identity.value = response.getIdentity()
    }
    onMounted(async () => {
      try {
        const response = await getUpdateIdentityRequestDetailFunc(userAccountId)
        if (!(response instanceof GetUpdateIdentityRequestDetailResp)) {
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
        await getIdentity(userAccountId)
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const approveReq = async () => {
      try {
        const response = await postUpdateIdentityRequestApprovalFunc(parseInt(userAccountId))
        if (!(response instanceof PostUpdateIdentityRequestApprovalResp)) {
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
        await router.push('/update-identity-request-approval')
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const chooseRejectionReason = async () => {
      const route = { name: 'UpdateIdentityRequestRejectionDetailPage', params: { user_account_id: userAccountId } }
      await router.push(route)
    }

    return { error, detail, image1Url, image2Url, identity, waitingRequestDone, approveReq, chooseRejectionReason }
  }
})
</script>
