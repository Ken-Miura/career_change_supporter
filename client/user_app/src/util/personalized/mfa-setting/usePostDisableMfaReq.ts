import { ref } from 'vue'
import { postDisableMfaReq } from './PostDisableMfaReq'

// eslint-disable-next-line
export function usePostDisableMfaReq () {
  const postDisableMfaReqDone = ref(true)
  const postDisableMfaReqFunc = async () => {
    try {
      postDisableMfaReqDone.value = false
      const response = await postDisableMfaReq()
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
