import { ref } from 'vue'
import { getLeftAwaitingWithdrawalByConsultationId } from './GetLeftAwaitingWithdrawalByConsultationId'

export function useGetLeftAwaitingWithdrawalByConsultationId () {
  const getLeftAwaitingWithdrawalByConsultationIdDone = ref(true)
  const getLeftAwaitingWithdrawalByConsultationIdFunc = async (consultationId: string) => {
    try {
      getLeftAwaitingWithdrawalByConsultationIdDone.value = false
      const response = await getLeftAwaitingWithdrawalByConsultationId(consultationId)
      return response
    } finally {
      getLeftAwaitingWithdrawalByConsultationIdDone.value = true
    }
  }
  return {
    getLeftAwaitingWithdrawalByConsultationIdDone,
    getLeftAwaitingWithdrawalByConsultationIdFunc
  }
}
