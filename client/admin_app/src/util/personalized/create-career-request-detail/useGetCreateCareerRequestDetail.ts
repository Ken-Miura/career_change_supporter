import { ref } from 'vue'
import { getCreateCareerRequestDetail } from './GetCreateCareerRequestDetail'

export function useGetCreateCareerRequestDetail () {
  const waitingGetCreateCareerRequestDetailDone = ref(false)
  const getCreateCareerRequestDetailFunc = async (createCreerReqId: string) => {
    try {
      waitingGetCreateCareerRequestDetailDone.value = true
      const response = await getCreateCareerRequestDetail(createCreerReqId)
      return response
    } finally {
      waitingGetCreateCareerRequestDetailDone.value = false
    }
  }
  return {
    waitingGetCreateCareerRequestDetailDone,
    getCreateCareerRequestDetailFunc
  }
}
