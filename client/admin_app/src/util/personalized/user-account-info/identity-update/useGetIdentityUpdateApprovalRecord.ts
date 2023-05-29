import { ref } from 'vue'
import { getIdentityUpdateApprovalRecord } from './GetIdentityUpdateApprovalRecord'

export function useGetIdentityUpdateApprovalRecord () {
  const getIdentityUpdateApprovalRecordDone = ref(true)
  const getIdentityUpdateApprovalRecordFunc = async (userAccountId: string) => {
    try {
      getIdentityUpdateApprovalRecordDone.value = false
      const response = await getIdentityUpdateApprovalRecord(userAccountId)
      return response
    } finally {
      getIdentityUpdateApprovalRecordDone.value = true
    }
  }
  return {
    getIdentityUpdateApprovalRecordDone,
    getIdentityUpdateApprovalRecordFunc
  }
}
