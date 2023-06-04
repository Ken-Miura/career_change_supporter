import { ref } from 'vue'
import { postEnableUserAccountReq } from './PostEnableUserAccountReq'

export function usePostEnableUserAccountReq () {
  const postEnableUserAccountReqDone = ref(true)
  const postEnableUserAccountReqFunc = async (userAccountId: number) => {
    try {
      postEnableUserAccountReqDone.value = false
      const response = await postEnableUserAccountReq(userAccountId)
      return response
    } finally {
      postEnableUserAccountReqDone.value = true
    }
  }
  return {
    postEnableUserAccountReqDone,
    postEnableUserAccountReqFunc
  }
}
