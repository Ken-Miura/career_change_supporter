import { ref } from 'vue'
import { postResumeSettlementReq } from './PostResumeSettlementReq'

export function usePostResumeSettlementReq () {
  const postResumeSettlementReqDone = ref(true)
  const postResumeSettlementReqFunc = async (stoppedsettlementId: number) => {
    try {
      postResumeSettlementReqDone.value = false
      const response = await postResumeSettlementReq(stoppedsettlementId)
      return response
    } finally {
      postResumeSettlementReqDone.value = true
    }
  }
  return {
    postResumeSettlementReqDone,
    postResumeSettlementReqFunc
  }
}
