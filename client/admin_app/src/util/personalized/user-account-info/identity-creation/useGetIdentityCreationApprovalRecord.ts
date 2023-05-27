import { ref } from 'vue'
import { getIdentityCreationApprovalRecord } from './GetIdentityCreationApprovalRecord'

export function useGetIdentityCreationApprovalRecord () {
  const getIdentityCreationApprovalRecordDone = ref(true)
  const getIdentityCreationApprovalRecordFunc = async (userAccountId: string) => {
    try {
      getIdentityCreationApprovalRecordDone.value = false
      const response = await getIdentityCreationApprovalRecord(userAccountId)
      return response
    } finally {
      getIdentityCreationApprovalRecordDone.value = true
    }
  }
  return {
    getIdentityCreationApprovalRecordDone,
    getIdentityCreationApprovalRecordFunc
  }
}
