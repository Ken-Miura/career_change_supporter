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
        <div data-test="identity" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">本人情報</h3>
          <p class="mt-2 text-lg">氏名が証明書画像の内容と一致しているか確認してください。</p>
          <div v-if="identity !== null">
            <div class="m-4 text-2xl grid grid-cols-3">
              <div class="mt-2 justify-self-start col-span-1">氏名</div><div class="justify-self-start col-span-2">{{ identity.last_name }} {{ identity.first_name }}</div>
              <div class="mt-2 justify-self-start col-span-1">フリガナ</div><div class="justify-self-start col-span-2">{{ identity.last_name_furigana }} {{ identity.first_name_furigana }}</div>
              <div class="mt-2 justify-self-start col-span-1">生年月日</div><div class="justify-self-start col-span-2">{{ identity.date_of_birth.year }}年{{ identity.date_of_birth.month }}月{{ identity.date_of_birth.day }}日</div>
              <div class="mt-2 justify-self-start col-span-3">住所</div>
              <div class="mt-2 ml-3 justify-self-start col-span-1">都道府県</div><div class="justify-self-start col-span-2">{{ identity.prefecture }}</div>
              <div class="mt-2 ml-3 justify-self-start col-span-1">市区町村</div><div class="justify-self-start col-span-2">{{ identity.city }}</div>
              <div class="mt-2 ml-3 justify-self-start col-span-1">番地</div><div class="justify-self-start col-span-2">{{ identity.address_line1 }}</div>
              <div v-if="identity.address_line2 !== null" class="mt-2 ml-3 justify-self-start col-span-1">建物名・部屋番号</div><div v-if="identity.address_line2 !== null" class="justify-self-start col-span-2">{{ identity.address_line2 }}</div>
              <div class="mt-2 justify-self-start col-span-1">電話番号</div><div class="justify-self-start col-span-2">{{ identity.telephone_number }}</div>
            </div>
          </div>
        </div>
        <div data-test="req-detail" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">職務経歴確認依頼詳細</h3>
          <div v-if="detail !== null">
            <div class="m-4 text-2xl grid grid-cols-3">
              <div class="mt-2 justify-self-start col-span-1">勤務先名称</div><div class="justify-self-start col-span-2">{{ detail.company_name }}</div>
              <div v-if="detail.department_name !== null" class="mt-2 ml-3 justify-self-start col-span-1">部署名</div><div v-if="detail.department_name !== null" class="justify-self-start col-span-2">{{ detail.department_name }}</div>
              <div v-if="detail.office !== null" class="mt-2 ml-3 justify-self-start col-span-1">勤務地</div><div v-if="detail.office !== null" class="justify-self-start col-span-2">{{ detail.office }}</div>
              <div class="mt-2 justify-self-start col-span-1">入社日</div><div class="justify-self-start col-span-2">{{ detail.career_start_date.year }}年{{ detail.career_start_date.month }}月{{ detail.career_start_date.day }}日</div>
              <div v-if="detail.career_end_date !== null" class="mt-2 justify-self-start col-span-1">退社日</div><div v-if="detail.career_end_date !== null" class="justify-self-start col-span-2">{{ detail.career_end_date.year }}年{{ detail.career_end_date.month }}月{{ detail.career_end_date.day }}日</div>
              <div class="mt-2 justify-self-start col-span-1">雇用形態</div>
              <div v-if="detail.contract_type === 'regular'" class="justify-self-start col-span-2">正社員</div>
              <div v-else-if="detail.contract_type === 'contract'" class="justify-self-start col-span-2">契約社員</div>
              <div v-else-if="detail.contract_type === 'other'" class="justify-self-start col-span-2">その他</div>
              <div v-else class="justify-self-start col-span-2">想定外の値です。管理者にご連絡下さい</div>
              <div v-if="detail.profession !== null" class="mt-2 ml-3 justify-self-start col-span-1">職種</div><div v-if="detail.profession !== null" class="justify-self-start col-span-2">{{ detail.profession }}</div>
              <div v-if="detail.annual_income_in_man_yen !== null" class="mt-2 ml-3 justify-self-start col-span-1">年収（単位：万円）</div><div v-if="detail.annual_income_in_man_yen !== null" class="justify-self-start col-span-2">{{ detail.annual_income_in_man_yen }}</div>
              <div class="mt-2 justify-self-start col-span-1">管理職区分</div>
              <div v-if="detail.is_manager" class="justify-self-start col-span-2">管理職</div>
              <div v-else class="justify-self-start col-span-2">非管理職</div>
              <div v-if="detail.position_name !== null" class="mt-2 ml-3 justify-self-start col-span-1">職位</div><div v-if="detail.position_name !== null" class="justify-self-start col-span-2">{{ detail.position_name }}</div>
              <div class="mt-2 justify-self-start col-span-1">入社区分</div>
              <div v-if="detail.is_new_graduate" class="justify-self-start col-span-2">新卒入社</div>
              <div v-else class="justify-self-start col-span-2">中途入社</div>
              <div v-if="detail.note !== null" class="mt-2 ml-3 justify-self-start col-span-1">備考</div><div v-if="detail.note !== null" class="justify-self-start col-span-2">{{ detail.note }}</div>
            </div>
            <div class="m-2 text-2xl">
              <div class="mt-2">証明書画像（表面）</div>
              <img data-test="req-detail-image1" class="mt-2" v-bind:src="image1Url" />
            </div>
            <div v-if="image2Url !== null" class="m-2 text-2xl">
              <div class="mt-2">証明書画像（裏面）</div>
              <img data-test="req-detail-image2" class="mt-2" v-bind:src="image2Url" />
            </div>
          </div>
        </div>
        <div class="flex flex-row justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
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
import { usePostCreateCareerRequestApproval } from '@/util/personalized/create-career-request-detail/usePostCreateCareerRequestApproval'
import { Code, createErrorMessage } from '@/util/Error'
import { ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'
import { Identity } from '@/util/personalized/Identity'
import { GetIdentityByUserAccountIdResp } from '@/util/personalized/GetIdentityByUserAccountIdResp'
import { useGetIdentityByUserAccountId } from '@/util/personalized/useGetIdentityByUserAccountId'
import { GetCreateCareerRequestDetailResp } from '@/util/personalized/create-career-request-detail/GetCreateCareerRequestDetailResp'
import { useGetCreateCareerRequestDetail } from '@/util/personalized/create-career-request-detail/useGetCreateCareerRequestDetail'
import { CreateCareerRequestDetail } from '@/util/personalized/create-career-request-detail/CreateCareerRequestDetail'
import { PostCreateCareerRequestApprovalResp } from '@/util/personalized/create-career-request-detail/PostCreateCareerRequestApprovalResp'

export default defineComponent({
  name: 'CreateCareerRequestDetailPage',
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
    const detail = ref(null as CreateCareerRequestDetail | null)
    const identity = ref(null as Identity | null)
    const route = useRoute()
    const router = useRouter()
    const createCareerReqId = route.params.create_career_req_id as string
    const image1Url = computed(() => {
      if (detail.value === null) {
        return ''
      }
      return `/admin/api/career-images/${detail.value.user_account_id}/${detail.value.image1_file_name_without_ext}`
    })
    const image2Url = computed(() => {
      if (detail.value === null) {
        return null
      }
      const image2Name = detail.value.image2_file_name_without_ext
      if (image2Name === null) {
        return null
      }
      return `/admin/api/career-images/${detail.value.user_account_id}/${image2Name}`
    })
    const {
      waitingGetCreateCareerRequestDetailDone,
      getCreateCareerRequestDetailFunc
    } = useGetCreateCareerRequestDetail()
    const {
      waitingGetIdentityByUserAccountIdDone,
      getIdentityByUserAccountIdFunc
    } = useGetIdentityByUserAccountId()
    const {
      waitingPostCreateCareerRequestApprovalDone,
      postCreateCareerRequestApprovalFunc
    } = usePostCreateCareerRequestApproval()
    const waitingRequestDone = computed(() => {
      return waitingGetCreateCareerRequestDetailDone.value || waitingGetIdentityByUserAccountIdDone.value || waitingPostCreateCareerRequestApprovalDone.value
    })
    const getIdentity = async (userAccoundId: string) => {
      const response = await getIdentityByUserAccountIdFunc(userAccoundId)
      if (!(response instanceof GetIdentityByUserAccountIdResp)) {
        if (!(response instanceof ApiErrorResp)) {
          throw new Error(`unexpected result on getting identity: ${response}`)
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
        const response = await getCreateCareerRequestDetailFunc(createCareerReqId)
        if (!(response instanceof GetCreateCareerRequestDetailResp)) {
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
        if (detail.value === null) {
          throw new Error(`create career request detail is null: ${response}`)
        }
        await getIdentity(detail.value.user_account_id.toString())
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const approveReq = async () => {
      try {
        const response = await postCreateCareerRequestApprovalFunc(parseInt(createCareerReqId))
        if (!(response instanceof PostCreateCareerRequestApprovalResp)) {
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
        await router.push('/create-career-request-approval')
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const chooseRejectionReason = async () => {
      const route = { name: 'CreateCareerRequestRejectionDetailPage', params: { create_career_req_id: createCareerReqId } }
      await router.push(route)
    }

    return { error, detail, image1Url, image2Url, identity, waitingRequestDone, approveReq, chooseRejectionReason }
  }
})
</script>
