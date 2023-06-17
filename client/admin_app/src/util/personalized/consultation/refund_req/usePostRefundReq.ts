import { ref } from 'vue'
import { postRefundReq } from './PostRefundReq'

export function usePostRefundReq () {
  const postRefundReqDone = ref(true)
  const postRefundReqFunc = async (settlementId: number) => {
    try {
      postRefundReqDone.value = false
      const response = await postRefundReq(settlementId)
      return response
    } finally {
      postRefundReqDone.value = true
    }
  }
  return {
    postRefundReqDone,
    postRefundReqFunc
  }
}
