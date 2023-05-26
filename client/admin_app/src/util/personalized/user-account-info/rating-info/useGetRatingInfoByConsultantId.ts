import { ref } from 'vue'
import { getRatingInfoByConsultantId } from './GetRatingInfoByConsultantId'

export function useGetRatingInfoByConsultantId () {
  const getRatingInfoByConsultantIdDone = ref(true)
  const getRatingInfoByConsultantIdFunc = async (consultantId: string) => {
    try {
      getRatingInfoByConsultantIdDone.value = false
      const response = await getRatingInfoByConsultantId(consultantId)
      return response
    } finally {
      getRatingInfoByConsultantIdDone.value = true
    }
  }
  return {
    getRatingInfoByConsultantIdDone,
    getRatingInfoByConsultantIdFunc
  }
}
