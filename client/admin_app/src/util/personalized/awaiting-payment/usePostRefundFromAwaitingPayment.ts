import { ref } from 'vue'
import { postRefundFromAwaitingPayment } from './PostRefundFromAwaitingPayment'

export function usePostRefundFromAwaitingPayment () {
  const postRefundFromAwaitingPaymentDone = ref(true)
  const postRefundFromAwaitingPaymentFunc = async (consultationId: number) => {
    try {
      postRefundFromAwaitingPaymentDone.value = false
      const response = await postRefundFromAwaitingPayment(consultationId)
      return response
    } finally {
      postRefundFromAwaitingPaymentDone.value = true
    }
  }
  return {
    postRefundFromAwaitingPaymentDone,
    postRefundFromAwaitingPaymentFunc
  }
}
