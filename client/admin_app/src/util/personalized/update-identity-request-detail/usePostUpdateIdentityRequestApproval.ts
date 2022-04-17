import { ref } from 'vue'
import { postUpdateIdentityRequestApproval } from './PostUpdateIdentityRequestApproval'

export function usePostUpdateIdentityRequestApproval () {
  const waitingPostUpdateIdentityRequestApprovalDone = ref(false)
  const postUpdateIdentityRequestApprovalFunc = async (userAccountId: number) => {
    try {
      waitingPostUpdateIdentityRequestApprovalDone.value = true
      const response = await postUpdateIdentityRequestApproval(userAccountId)
      return response
    } finally {
      waitingPostUpdateIdentityRequestApprovalDone.value = false
    }
  }
  return {
    waitingPostUpdateIdentityRequestApprovalDone,
    postUpdateIdentityRequestApprovalFunc
  }
}
