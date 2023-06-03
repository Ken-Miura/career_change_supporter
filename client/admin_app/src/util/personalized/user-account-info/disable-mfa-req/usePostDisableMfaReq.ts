import { ref } from 'vue'
import { postDisableMfaReq } from './PostDisableMfaReq'

export function usePostDisableMfaReq () {
  const postDisableMfaReqDone = ref(true)
  const postDisableMfaReqFunc = async (userAccountId: number) => {
    try {
      postDisableMfaReqDone.value = false
      const response = await postDisableMfaReq(userAccountId)
      return response
    } finally {
      postDisableMfaReqDone.value = true
    }
  }
  return {
    postDisableMfaReqDone,
    postDisableMfaReqFunc
  }
}
