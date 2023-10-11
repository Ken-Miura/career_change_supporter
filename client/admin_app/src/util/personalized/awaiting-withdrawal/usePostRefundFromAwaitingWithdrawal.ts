import { ref } from 'vue'
import { postRefundFromAwaitingWithdrawal } from './PostRefundFromAwaitingWithdrawal'

export function usePostRefundFromAwaitingWithdrawal () {
  const postRefundFromAwaitingWithdrawalDone = ref(true)
  const postRefundFromAwaitingWithdrawalFunc = async (consultationId: number) => {
    try {
      postRefundFromAwaitingWithdrawalDone.value = false
      const response = await postRefundFromAwaitingWithdrawal(consultationId)
      return response
    } finally {
      postRefundFromAwaitingWithdrawalDone.value = true
    }
  }
  return {
    postRefundFromAwaitingWithdrawalDone,
    postRefundFromAwaitingWithdrawalFunc
  }
}
