import { ref } from 'vue'
import { getConsultationRequests } from './GetConsultationRequests'

export function useGetConsultationRequests () {
  const getConsultationRequestsDone = ref(true)
  const getConsultationRequestsFunc = async () => {
    try {
      getConsultationRequestsDone.value = false
      const response = await getConsultationRequests()
      return response
    } finally {
      getConsultationRequestsDone.value = true
    }
  }
  return {
    getConsultationRequestsDone,
    getConsultationRequestsFunc
  }
}
