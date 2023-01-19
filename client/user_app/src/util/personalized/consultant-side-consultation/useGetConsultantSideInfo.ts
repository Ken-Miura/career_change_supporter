import { ref } from 'vue'
import { getConsultantSideInfo } from './GetConsultantSideInfo'

// eslint-disable-next-line
export function useGetConsultantSideInfo () {
  const getConsultantSideInfoDone = ref(true)
  const getConsultantSideInfoFunc = async (consultationId: string) => {
    try {
      getConsultantSideInfoDone.value = false
      const response = await getConsultantSideInfo(consultationId)
      return response
    } finally {
      getConsultantSideInfoDone.value = true
    }
  }
  return {
    getConsultantSideInfoDone,
    getConsultantSideInfoFunc
  }
}
