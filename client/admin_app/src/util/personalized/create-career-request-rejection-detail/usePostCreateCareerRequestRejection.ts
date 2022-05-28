import { ref } from 'vue'
import { postCreateCareerRequestRejection } from './PostCreateCareerRequestRejection'

export function usePostCreateCareerRequestRejection () {
  const waitingRequestDone = ref(false)
  const postCreateCareerRequestRejectionFunc = async (createCareerReqId: number, rejectionReason: string) => {
    try {
      waitingRequestDone.value = true
      const response = await postCreateCareerRequestRejection(createCareerReqId, rejectionReason)
      return response
    } finally {
      waitingRequestDone.value = false
    }
  }
  return {
    waitingRequestDone,
    postCreateCareerRequestRejectionFunc
  }
}
