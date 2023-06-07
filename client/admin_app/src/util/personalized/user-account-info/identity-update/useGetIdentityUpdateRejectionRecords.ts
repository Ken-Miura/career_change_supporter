import { ref } from 'vue'
import { getIdentityUpdateRejectionRecords } from './GetIdentityUpdateRejectionRecords'

export function useGetIdentityUpdateRejectionRecords () {
  const getIdentityUpdateRejectionRecordsDone = ref(true)
  const getIdentityUpdateRejectionRecordsFunc = async (userAccountId: string) => {
    try {
      getIdentityUpdateRejectionRecordsDone.value = false
      const response = await getIdentityUpdateRejectionRecords(userAccountId)
      return response
    } finally {
      getIdentityUpdateRejectionRecordsDone.value = true
    }
  }
  return {
    getIdentityUpdateRejectionRecordsDone,
    getIdentityUpdateRejectionRecordsFunc
  }
}
