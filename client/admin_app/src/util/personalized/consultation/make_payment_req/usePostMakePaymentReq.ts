import { ref } from 'vue'
import { postMakePaymentReq } from './PostMakePaymentReq'

export function usePostMakePaymentReq () {
  const postMakePaymentReqDone = ref(true)
  const postMakePaymentReqFunc = async (settlementId: number) => {
    try {
      postMakePaymentReqDone.value = false
      const response = await postMakePaymentReq(settlementId)
      return response
    } finally {
      postMakePaymentReqDone.value = true
    }
  }
  return {
    postMakePaymentReqDone,
    postMakePaymentReqFunc
  }
}
