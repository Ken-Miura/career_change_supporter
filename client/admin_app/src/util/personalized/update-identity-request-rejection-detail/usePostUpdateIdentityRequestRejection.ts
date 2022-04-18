import { ref } from 'vue'
import { postUpdateIdentityRequestRejection } from './PostUpdateIdentityRequestRejection'

export function usePostUpdateIdentityRequestRejection () {
  const waitingRequestDone = ref(false)
  const postUpdateIdentityRequestRejectionFunc = async (userAccountId: number, rejectionReason: string) => {
    try {
      waitingRequestDone.value = true
      const response = await postUpdateIdentityRequestRejection(userAccountId, rejectionReason)
      return response
    } finally {
      waitingRequestDone.value = false
    }
  }
  return {
    waitingRequestDone,
    postUpdateIdentityRequestRejectionFunc
  }
}
