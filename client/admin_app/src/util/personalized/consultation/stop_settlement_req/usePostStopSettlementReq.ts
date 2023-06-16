import { ref } from 'vue'
import { postStopSettlementReq } from './PostStopSettlementReq'

export function usePostStopSettlementReq () {
  const postStopSettlementReqDone = ref(true)
  const postStopSettlementReqFunc = async (settlementId: number) => {
    try {
      postStopSettlementReqDone.value = false
      const response = await postStopSettlementReq(settlementId)
      return response
    } finally {
      postStopSettlementReqDone.value = true
    }
  }
  return {
    postStopSettlementReqDone,
    postStopSettlementReqFunc
  }
}
