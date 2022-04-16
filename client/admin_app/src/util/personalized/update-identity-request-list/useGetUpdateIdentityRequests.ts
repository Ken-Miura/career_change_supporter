import { ref } from 'vue'
import { getUpdateIdentityRequests } from './GetUpdateIdentityRequests'

export function useGetUpdateIdentityRequests () {
  const waitingRequestDone = ref(false)
  const getUpdateIdentityRequestsFunc = async (page: number, perPage: number) => {
    try {
      waitingRequestDone.value = true
      const response = await getUpdateIdentityRequests(page, perPage)
      return response
    } finally {
      waitingRequestDone.value = false
    }
  }
  return {
    waitingRequestDone,
    getUpdateIdentityRequestsFunc
  }
}
