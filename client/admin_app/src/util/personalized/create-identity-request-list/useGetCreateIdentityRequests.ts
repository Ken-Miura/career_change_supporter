import { ref } from 'vue'
import { getCreateIdentityRequests } from './GetCreateIdentityRequests'

export function useGetCreateIdentityRequests () {
  const waitingRequestDone = ref(false)
  const getCreateIdentityRequestsFunc = async (page: number, perPage: number) => {
    try {
      waitingRequestDone.value = true
      const response = await getCreateIdentityRequests(page, perPage)
      return response
    } finally {
      waitingRequestDone.value = false
    }
  }
  return {
    waitingRequestDone,
    getCreateIdentityRequestsFunc
  }
}
