import { ref } from 'vue'
import { getCareerCreationApprovalRecords } from './GetCareerCreationApprovalRecords'

export function useGetCareerCreationApprovalRecords () {
  const getCareerCreationApprovalRecordsDone = ref(true)
  const getCareerCreationApprovalRecordsFunc = async (userAccountId: string) => {
    try {
      getCareerCreationApprovalRecordsDone.value = false
      const response = await getCareerCreationApprovalRecords(userAccountId)
      return response
    } finally {
      getCareerCreationApprovalRecordsDone.value = true
    }
  }
  return {
    getCareerCreationApprovalRecordsDone,
    getCareerCreationApprovalRecordsFunc
  }
}
