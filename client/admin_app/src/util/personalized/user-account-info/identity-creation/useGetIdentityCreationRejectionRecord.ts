import { ref } from 'vue'
import { getIdentityCreationRejectionRecord } from './GetIdentityCreationRejectionRecord'

export function useGetIdentityCreationRejectionRecord () {
  const getIdentityCreationRejectionRecordDone = ref(true)
  const getIdentityCreationRejectionRecordFunc = async (userAccountId: string) => {
    try {
      getIdentityCreationRejectionRecordDone.value = false
      const response = await getIdentityCreationRejectionRecord(userAccountId)
      return response
    } finally {
      getIdentityCreationRejectionRecordDone.value = true
    }
  }
  return {
    getIdentityCreationRejectionRecordDone,
    getIdentityCreationRejectionRecordFunc
  }
}
