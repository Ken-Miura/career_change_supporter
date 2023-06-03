import { ref } from 'vue'
import { postDisableUserAccountReq } from './PostDisableUserAccountReq'

export function usePostDisableUserAccountReq () {
  const postDisableUserAccountReqDone = ref(true)
  const postDisableUserAccountReqFunc = async (userAccountId: number) => {
    try {
      postDisableUserAccountReqDone.value = false
      const response = await postDisableUserAccountReq(userAccountId)
      return response
    } finally {
      postDisableUserAccountReqDone.value = true
    }
  }
  return {
    postDisableUserAccountReqDone,
    postDisableUserAccountReqFunc
  }
}
