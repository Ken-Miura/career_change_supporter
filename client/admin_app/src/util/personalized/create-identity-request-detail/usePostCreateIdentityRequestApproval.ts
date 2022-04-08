import { ref } from 'vue'
import { postCreateIdentityRequestApproval } from './PostCreateIdentityRequestApproval'

export function usePostCreateIdentityRequestApproval () {
  const waitingPostCreateIdentityRequestApprovalDone = ref(false)
  const postCreateIdentityRequestApprovalFunc = async (userAccountId: number) => {
    try {
      waitingPostCreateIdentityRequestApprovalDone.value = true
      const response = await postCreateIdentityRequestApproval(userAccountId)
      return response
    } finally {
      waitingPostCreateIdentityRequestApprovalDone.value = false
    }
  }
  return {
    waitingPostCreateIdentityRequestApprovalDone,
    postCreateIdentityRequestApprovalFunc
  }
}
