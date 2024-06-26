<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getCareerDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="error.exists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="error.message"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-xl lg:text-2xl">職務経歴</h3>
          <div v-if="career !== null" data-test="career-set" class="m-4 text-base lg:text-2xl grid grid-cols-2">
            <div class="mt-2 justify-self-start col-span-1">勤務先名称</div><div class="mt-2 justify-self-start col-span-1">{{ career.company_name }}</div>
            <div v-if="career.department_name !== null" class="mt-2 justify-self-start col-span-1">部署名</div><div v-if="career.department_name !== null" class="mt-2 justify-self-start col-span-1">{{ career.department_name }}</div>
            <div v-if="career.office !== null" class="mt-2 justify-self-start col-span-1">勤務地</div><div v-if="career.office !== null" class="mt-2 justify-self-start col-span-1">{{ career.office }}</div>
            <div class="mt-2 justify-self-start col-span-1">入社日</div><div class="mt-2 justify-self-start col-span-1">{{ career.career_start_date.year }}年{{ career.career_start_date.month }}月{{ career.career_start_date.day }}日</div>
            <div v-if="career.career_end_date !== null" class="mt-2 justify-self-start col-span-1">退社日</div><div v-if="career.career_end_date !== null" class="mt-2 justify-self-start col-span-1">{{ career.career_end_date.year }}年{{ career.career_end_date.month }}月{{ career.career_end_date.day }}日</div>
            <div class="mt-2 justify-self-start col-span-1">雇用形態</div>
            <div v-if="career.contract_type === 'regular'" class="mt-2 justify-self-start col-span-1">正社員</div>
            <div v-else-if="career.contract_type === 'contract'" class="mt-2 justify-self-start col-span-1">契約社員</div>
            <div v-else-if="career.contract_type === 'other'" class="mt-2 justify-self-start col-span-1">その他</div>
            <div v-else class="mt-2 justify-self-start col-span-1">想定外の値です。管理者にご連絡下さい</div>
            <div v-if="career.profession !== null" class="mt-2 justify-self-start col-span-1">職種</div><div v-if="career.profession !== null" class="mt-2 justify-self-start col-span-1">{{ career.profession }}</div>
            <div v-if="career.annual_income_in_man_yen !== null" class="mt-2 justify-self-start col-span-1">年収（単位：万円）</div><div v-if="career.annual_income_in_man_yen !== null" class="mt-2 justify-self-start col-span-1">{{ career.annual_income_in_man_yen }}</div>
            <div class="mt-2 justify-self-start col-span-1">管理職区分</div>
            <div v-if="career.is_manager" class="mt-2 justify-self-start col-span-1">管理職</div>
            <div v-else class="mt-2 justify-self-start col-span-1">非管理職</div>
            <div v-if="career.position_name !== null" class="mt-2 justify-self-start col-span-1">職位</div><div v-if="career.position_name !== null" class="mt-2 justify-self-start col-span-1">{{ career.position_name }}</div>
            <div class="mt-2 justify-self-start col-span-1">入社区分</div>
            <div v-if="career.is_new_graduate" class="mt-2 justify-self-start col-span-1">新卒入社</div>
            <div v-else class="mt-2 justify-self-start col-span-1">中途入社</div>
            <div v-if="career.note !== null" class="mt-2 justify-self-start col-span-2">備考</div>
            <div v-if="career.note !== null" class="mt-1 ml-1 justify-self-start col-span-2 whitespace-pre-wrap">{{ career.note }}</div>
          </div>
          <p v-else data-test="no-career-set" class="m-4 text-base lg:text-xl">職務経歴を取得出来ませんでした。</p>
          <button data-test="move-to-career-deletion-confirm-page-button" v-on:click="moveToCareerDeletionConfirmPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">職務経歴の削除</button>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { useGetCareer } from '@/util/personalized/career-detail/useGetCareer'
import { Message } from '@/util/Message'
import { GetCareerResp } from '@/util/personalized/career-detail/GetCareerResp'
import { Career } from '@/util/personalized/Career'

export default defineComponent({
  name: 'CareerDetailPage',
  components: {
    TheHeader,
    WaitingCircle,
    AlertMessage
  },
  setup () {
    const error = reactive({
      exists: false,
      message: ''
    })
    const career = ref(null as Career | null)
    const route = useRoute()
    const careerId = route.params.career_id as string
    const router = useRouter()
    const {
      getCareerDone,
      getCareerFunc
    } = useGetCareer()

    onMounted(async () => {
      try {
        const response = await getCareerFunc(parseInt(careerId))
        if (!(response instanceof GetCareerResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          }
          error.exists = true
          error.message = createErrorMessage(response.getApiError().getCode())
          return
        }
        career.value = response.getCareer()
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const moveToCareerDeletionConfirmPage = async () => {
      const route = JSON.parse(`{"name": "CareerDeletionConfirmPage", "params": {"career_id": ${careerId}}}`)
      await router.push(route)
    }

    return { error, career, getCareerDone, moveToCareerDeletionConfirmPage }
  }
})
</script>
