import { ref } from 'vue'
import { getIdentityUpdateApprovalRecords } from './GetIdentityUpdateApprovalRecords'

export function useGetIdentityUpdateApprovalRecords () {
  const getIdentityUpdateApprovalRecordsDone = ref(true)
  const getIdentityUpdateApprovalRecordsFunc = async (userAccountId: string) => {
    try {
      getIdentityUpdateApprovalRecordsDone.value = false
      const response = await getIdentityUpdateApprovalRecords(userAccountId)
      return response
    } finally {
      getIdentityUpdateApprovalRecordsDone.value = true
    }
  }
  return {
    getIdentityUpdateApprovalRecordsDone,
    getIdentityUpdateApprovalRecordsFunc
  }
}
