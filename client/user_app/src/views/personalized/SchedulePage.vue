<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg text-center">getConsultationsDone {{ getConsultationsDone }}</h3>
      <h3 class="font-bold text-lg text-center">-----</h3>
      <h3 class="font-bold text-lg text-center">consultationsResult {{ consultationsResult }}</h3>
      <h3 class="font-bold text-lg text-center">-----</h3>
      <h3 class="font-bold text-lg text-center">error {{ error }}</h3>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { Message } from '@/util/Message'
import { useGetConsultations } from '@/util/personalized/schedule/useGetConsultations'
import { GetConsultationsResp } from '@/util/personalized/schedule/GetConsultationsResp'
import { ConsultationsResult } from '@/util/personalized/schedule/ConsultationsResult'

export default defineComponent({
  name: 'SchedulePage',
  components: {
    TheHeader
  },
  setup () {
    const {
      getConsultationsDone,
      getConsultationsFunc
    } = useGetConsultations()
    const consultationsResult = ref({ user_side_consultations: [], consultant_side_consultations: [] } as ConsultationsResult)
    const error = reactive({
      exists: false,
      message: ''
    })
    const router = useRouter()
    onMounted(async () => {
      try {
        const resp = await getConsultationsFunc()
        if (!(resp instanceof GetConsultationsResp)) {
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
          error.exists = true
          error.message = createErrorMessage(resp.getApiError().getCode())
          return
        }
        consultationsResult.value = resp.getConsultationsResult()
      } catch (e) {
        error.exists = true
        error.message = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })
    return {
      getConsultationsDone,
      consultationsResult,
      error
    }
  }
})
</script>
