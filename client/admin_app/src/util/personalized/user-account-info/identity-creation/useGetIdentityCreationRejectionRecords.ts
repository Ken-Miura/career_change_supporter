import { ref } from 'vue'
import { getIdentityCreationRejectionRecords } from './GetIdentityCreationRejectionRecords'

export function useGetIdentityCreationRejectionRecords () {
  const getIdentityCreationRejectionRecordsDone = ref(true)
  const getIdentityCreationRejectionRecordsFunc = async (userAccountId: string) => {
    try {
      getIdentityCreationRejectionRecordsDone.value = false
      const response = await getIdentityCreationRejectionRecords(userAccountId)
      return response
    } finally {
      getIdentityCreationRejectionRecordsDone.value = true
    }
  }
  return {
    getIdentityCreationRejectionRecordsDone,
    getIdentityCreationRejectionRecordsFunc
  }
}
