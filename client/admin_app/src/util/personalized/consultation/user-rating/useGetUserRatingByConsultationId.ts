import { ref } from 'vue'
import { getUserRatingByConsultationId } from './GetUserRatingByConsultationId'

export function useGetUserRatingByConsultationId () {
  const getUserRatingByConsultationIdDone = ref(true)
  const getUserRatingByConsultationIdFunc = async (consultationId: string) => {
    try {
      getUserRatingByConsultationIdDone.value = false
      const response = await getUserRatingByConsultationId(consultationId)
      return response
    } finally {
      getUserRatingByConsultationIdDone.value = true
    }
  }
  return {
    getUserRatingByConsultationIdDone,
    getUserRatingByConsultationIdFunc
  }
}
