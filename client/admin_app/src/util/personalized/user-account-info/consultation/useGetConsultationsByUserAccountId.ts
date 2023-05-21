import { ref } from 'vue'
import { getConsultationsByUserAccountId } from './GetConsultationsByUserAccountId'

export function useGetConsultationsByUserAccountId () {
  const getConsultationsByUserAccountIdDone = ref(true)
  const getConsultationsByUserAccountIdFunc = async (userAccountId: string) => {
    try {
      getConsultationsByUserAccountIdDone.value = false
      const response = await getConsultationsByUserAccountId(userAccountId)
      return response
    } finally {
      getConsultationsByUserAccountIdDone.value = true
    }
  }
  return {
    getConsultationsByUserAccountIdDone,
    getConsultationsByUserAccountIdFunc
  }
}
