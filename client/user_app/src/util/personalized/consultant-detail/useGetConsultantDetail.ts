import { ref } from 'vue'
import { getConsultantDetail } from './GetConsultantDetail'

// eslint-disable-next-line
export function useGetConsultantDetail () {
  const getConsultantDetailDone = ref(true)
  const getConsultantDetailFunc = async (consultantId: string) => {
    try {
      getConsultantDetailDone.value = false
      const response = await getConsultantDetail(consultantId)
      return response
    } finally {
      getConsultantDetailDone.value = true
    }
  }
  return {
    getConsultantDetailDone,
    getConsultantDetailFunc
  }
}
