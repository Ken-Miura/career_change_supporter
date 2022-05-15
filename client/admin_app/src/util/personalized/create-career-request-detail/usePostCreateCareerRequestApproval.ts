import { ref } from 'vue'
import { postCreateCareerRequestApproval } from './PostCreateCareerRequestApproval'

export function usePostCreateCareerRequestApproval () {
  const waitingPostCreateCareerRequestApprovalDone = ref(false)
  const postCreateCareerRequestApprovalFunc = async (createCareerReqId: number) => {
    try {
      waitingPostCreateCareerRequestApprovalDone.value = true
      const response = await postCreateCareerRequestApproval(createCareerReqId)
      return response
    } finally {
      waitingPostCreateCareerRequestApprovalDone.value = false
    }
  }
  return {
    waitingPostCreateCareerRequestApprovalDone,
    postCreateCareerRequestApprovalFunc
  }
}
