import { ref } from 'vue'
import { getReceiptsOfConsultation } from './GetReceiptsOfConsultation'

export function useGetReceiptsOfConsultation () {
  const getReceiptsOfConsultationDone = ref(true)
  const getReceiptsOfConsultationFunc = async (page: number, perPage: number) => {
    try {
      getReceiptsOfConsultationDone.value = false
      const response = await getReceiptsOfConsultation(page, perPage)
      return response
    } finally {
      getReceiptsOfConsultationDone.value = true
    }
  }
  return {
    getReceiptsOfConsultationDone,
    getReceiptsOfConsultationFunc
  }
}
