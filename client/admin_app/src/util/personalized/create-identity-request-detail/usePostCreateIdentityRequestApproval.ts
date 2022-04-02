import { ref } from 'vue'
import { postCreateIdentityRequestApproval } from './PostCreateIdentityRequestApproval'

export function usePostCreateIdentityRequestApproval () {
  const waitingpostCreateIdentityRequestApprovalDone = ref(false)
  const postCreateIdentityRequestApprovalFunc = async (userAccountId: number) => {
    try {
      waitingpostCreateIdentityRequestApprovalDone.value = true
      const response = await postCreateIdentityRequestApproval(userAccountId)
      return response
    } finally {
      waitingpostCreateIdentityRequestApprovalDone.value = false
    }
  }
  return {
    waitingpostCreateIdentityRequestApprovalDone,
    postCreateIdentityRequestApprovalFunc
  }
}
