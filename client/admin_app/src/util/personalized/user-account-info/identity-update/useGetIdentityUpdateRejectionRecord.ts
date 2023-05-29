import { ref } from 'vue'
import { getIdentityUpdateRejectionRecord } from './GetIdentityUpdateRejectionRecord'

export function useGetIdentityUpdateRejectionRecord () {
  const getIdentityUpdateRejectionRecordDone = ref(true)
  const getIdentityUpdateRejectionRecordFunc = async (userAccountId: string) => {
    try {
      getIdentityUpdateRejectionRecordDone.value = false
      const response = await getIdentityUpdateRejectionRecord(userAccountId)
      return response
    } finally {
      getIdentityUpdateRejectionRecordDone.value = true
    }
  }
  return {
    getIdentityUpdateRejectionRecordDone,
    getIdentityUpdateRejectionRecordFunc
  }
}
