import { ref } from 'vue'
import { postEnableMfaReq } from './PostEnableMfaReq'

// eslint-disable-next-line
export function usePostEnableMfaReq () {
  const postEnableMfaReqDone = ref(true)
  const postEnableMfaReqFunc = async (passCode: string) => {
    try {
      postEnableMfaReqDone.value = false
      const response = await postEnableMfaReq(passCode)
      return response
    } finally {
      postEnableMfaReqDone.value = true
    }
  }
  return {
    postEnableMfaReqDone,
    postEnableMfaReqFunc
  }
}
