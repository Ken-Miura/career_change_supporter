import { ref } from 'vue'
import { getCreateCareerRequests } from './GetCreateCareerRequests'

export function useGetCreateCareerRequests () {
  const waitingRequestDone = ref(false)
  const getCreateCareerRequestsFunc = async (page: number, perPage: number) => {
    try {
      waitingRequestDone.value = true
      const response = await getCreateCareerRequests(page, perPage)
      return response
    } finally {
      waitingRequestDone.value = false
    }
  }
  return {
    waitingRequestDone,
    getCreateCareerRequestsFunc
  }
}
