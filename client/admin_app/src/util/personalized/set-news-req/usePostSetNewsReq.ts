import { ref } from 'vue'
import { postSetNewsReq } from './PostSetNewsReq'
import { SetNewsReq } from './SetNewsReq'

export function usePostSetNewsReq () {
  const postSetNewsReqDone = ref(true)
  const postSetNewsReqFunc = async (req: SetNewsReq) => {
    try {
      postSetNewsReqDone.value = false
      const response = await postSetNewsReq(req)
      return response
    } finally {
      postSetNewsReqDone.value = true
    }
  }
  return {
    postSetNewsReqDone,
    postSetNewsReqFunc
  }
}
