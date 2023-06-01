import { ref } from 'vue'
import { getCareerCreationApprovalRecord } from './GetCareerCreationApprovalRecord'

export function useGetCareerCreationApprovalRecord () {
  const getCareerCreationApprovalRecordDone = ref(true)
  const getCareerCreationApprovalRecordFunc = async (userAccountId: string) => {
    try {
      getCareerCreationApprovalRecordDone.value = false
      const response = await getCareerCreationApprovalRecord(userAccountId)
      return response
    } finally {
      getCareerCreationApprovalRecordDone.value = true
    }
  }
  return {
    getCareerCreationApprovalRecordDone,
    getCareerCreationApprovalRecordFunc
  }
}
