import { ref } from 'vue'
import { getCareerCreationRejectionRecords } from './GetCareerCreationRejectionRecords'

export function useGetCareerCreationRejectionRecords () {
  const getCareerCreationRejectionRecordsDone = ref(true)
  const getCareerCreationRejectionRecordsFunc = async (userAccountId: string) => {
    try {
      getCareerCreationRejectionRecordsDone.value = false
      const response = await getCareerCreationRejectionRecords(userAccountId)
      return response
    } finally {
      getCareerCreationRejectionRecordsDone.value = true
    }
  }
  return {
    getCareerCreationRejectionRecordsDone,
    getCareerCreationRejectionRecordsFunc
  }
}
