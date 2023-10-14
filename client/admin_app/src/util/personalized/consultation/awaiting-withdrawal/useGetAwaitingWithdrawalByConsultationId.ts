import { ref } from 'vue'
import { getAwaitingWithdrawalByConsultationId } from './GetAwaitingWithdrawalByConsultationId'

export function useGetAwaitingWithdrawalByConsultationId () {
  const getAwaitingWithdrawalByConsultationIdDone = ref(true)
  const getAwaitingWithdrawalByConsultationIdFunc = async (consultationId: string) => {
    try {
      getAwaitingWithdrawalByConsultationIdDone.value = false
      const response = await getAwaitingWithdrawalByConsultationId(consultationId)
      return response
    } finally {
      getAwaitingWithdrawalByConsultationIdDone.value = true
    }
  }
  return {
    getAwaitingWithdrawalByConsultationIdDone,
    getAwaitingWithdrawalByConsultationIdFunc
  }
}
