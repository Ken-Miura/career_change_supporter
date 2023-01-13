import { ref } from 'vue'
import { getConsultations } from './GetConsultations'

// eslint-disable-next-line
export function useGetConsultations () {
  const getConsultationsDone = ref(true)
  const getConsultationsFunc = async () => {
    try {
      getConsultationsDone.value = false
      const response = await getConsultations()
      return response
    } finally {
      getConsultationsDone.value = true
    }
  }
  return {
    getConsultationsDone,
    getConsultationsFunc
  }
}
