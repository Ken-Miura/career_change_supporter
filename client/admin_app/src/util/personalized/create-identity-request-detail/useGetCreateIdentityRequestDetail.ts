import { ref } from 'vue'
import { getCreateIdentityRequestDetail } from './GetCreateIdentityRequestDetail'

export function useGetCreateIdentityRequestDetail () {
  const waitingGetCreateIdentityRequestDetailDone = ref(false)
  const getCreateIdentityRequestDetailFunc = async (userAccountId: string) => {
    try {
      waitingGetCreateIdentityRequestDetailDone.value = true
      const response = await getCreateIdentityRequestDetail(userAccountId)
      return response
    } finally {
      waitingGetCreateIdentityRequestDetailDone.value = false
    }
  }
  return {
    waitingGetCreateIdentityRequestDetailDone,
    getCreateIdentityRequestDetailFunc
  }
}
