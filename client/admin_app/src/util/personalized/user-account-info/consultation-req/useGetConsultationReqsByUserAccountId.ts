import { ref } from 'vue'
import { getConsultationReqsByUserAccountId } from './GetConsultationReqsByUserAccountId'

export function useGetConsultationReqsByUserAccountId () {
  const getConsultationReqsByUserAccountIdDone = ref(true)
  const getConsultationReqsByUserAccountIdFunc = async (userAccountId: string) => {
    try {
      getConsultationReqsByUserAccountIdDone.value = false
      const response = await getConsultationReqsByUserAccountId(userAccountId)
      return response
    } finally {
      getConsultationReqsByUserAccountIdDone.value = true
    }
  }
  return {
    getConsultationReqsByUserAccountIdDone,
    getConsultationReqsByUserAccountIdFunc
  }
}
