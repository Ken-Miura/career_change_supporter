import { ref } from 'vue'
import { getCareerCreationRejectionRecord } from './GetCareerCreationRejectionRecord'

export function useGetCareerCreationRejectionRecord () {
  const getCareerCreationRejectionRecordDone = ref(true)
  const getCareerCreationRejectionRecordFunc = async (userAccountId: string) => {
    try {
      getCareerCreationRejectionRecordDone.value = false
      const response = await getCareerCreationRejectionRecord(userAccountId)
      return response
    } finally {
      getCareerCreationRejectionRecordDone.value = true
    }
  }
  return {
    getCareerCreationRejectionRecordDone,
    getCareerCreationRejectionRecordFunc
  }
}
