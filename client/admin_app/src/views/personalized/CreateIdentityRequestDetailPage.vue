<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="waitingGetCreateIdentityRequestDetailDone" class="m-6">
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
import { CreateIdentityRequestDetail } from '@/util/personalized/create-identity-request-detail/CreateIdentityRequestDetail'
import { useGetCreateIdentityRequestDetail } from '@/util/personalized/create-identity-request-detail/useGetCreateIdentityRequestDetail'
import { GetCreateIdentityRequestDetailResp } from '@/util/personalized/create-identity-request-detail/GetCreateIdentityRequestDetailResp'
import { Code, createErrorMessage } from '@/util/Error'
import { ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'

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
    onMounted(async () => {
      try {
        const response = await getCreateIdentityRequestDetailFunc(userAccountId)
        if (response instanceof GetCreateIdentityRequestDetailResp) {
          detail.value = response.getDetail()
        } else if (response instanceof ApiErrorResp) {
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          error.exists = true
          error.message = createErrorMessage(response.getApiError().getCode())
        } else {
          throw new Error(`unexpected result: ${response}`)
        }
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }

      // const params = { year: '1990', month: '3', day: '1' }
      // const query = new URLSearchParams(params)
      // const response = await fetch(`/admin/api/users-by-birthday?${query}`, {
      //   method: 'GET'
      // })
      // if (!response.ok) {
      //   const apiErr = await response.json() as { code: number }
      //   console.log(apiErr)
      //   return
      // }
      // const data = await response.json()
      // console.log(data)
    })
    return { error, detail, image1Url, image2Url, waitingGetCreateIdentityRequestDetailDone }
  }
})
</script>
