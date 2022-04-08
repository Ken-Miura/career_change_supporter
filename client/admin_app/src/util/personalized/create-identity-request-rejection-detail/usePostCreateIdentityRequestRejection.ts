import { ref } from 'vue'
import { postCreateIdentityRequestRejection } from './PostCreateIdentityRequestRejection'

export function usePostCreateIdentityRequestRejection () {
  const waitingRequestDone = ref(false)
  const postCreateIdentityRequestRejectionFunc = async (userAccountId: number, rejectionReason: string) => {
    try {
      waitingRequestDone.value = true
      const response = await postCreateIdentityRequestRejection(userAccountId, rejectionReason)
      return response
    } finally {
      waitingRequestDone.value = false
    }
  }
  return {
    waitingRequestDone,
    postCreateIdentityRequestRejectionFunc
  }
}
