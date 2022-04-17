import { ref } from 'vue'
import { getUpdateIdentityRequestDetail } from './GetUpdateIdentityRequestDetail'

export function useGetUpdateIdentityRequestDetail () {
  const waitingGetUpdateIdentityRequestDetailDone = ref(false)
  const getUpdateIdentityRequestDetailFunc = async (userAccountId: string) => {
    try {
      waitingGetUpdateIdentityRequestDetailDone.value = true
      const response = await getUpdateIdentityRequestDetail(userAccountId)
      return response
    } finally {
      waitingGetUpdateIdentityRequestDetailDone.value = false
    }
  }
  return {
    waitingGetUpdateIdentityRequestDetailDone,
    getUpdateIdentityRequestDetailFunc
  }
}
