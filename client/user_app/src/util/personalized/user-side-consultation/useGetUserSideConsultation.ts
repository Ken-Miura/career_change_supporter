import { ref } from 'vue'
import { getUserSideConsultation } from './GetUserSideConsultation'

// eslint-disable-next-line
export function useGetUserSideConsultation () {
  const getUserSideConsultationDone = ref(true)
  const getUserSideConsultationFunc = async (consultationId: string) => {
    try {
      getUserSideConsultationDone.value = false
      const response = await getUserSideConsultation(consultationId)
      return response
    } finally {
      getUserSideConsultationDone.value = true
    }
  }
  return {
    getUserSideConsultationDone,
    getUserSideConsultationFunc
  }
}
