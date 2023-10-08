import { ref } from 'vue'
import { postNeglectedPayment } from './PostNeglectedPayment'

export function usePostNeglectedPayment () {
  const postNeglectedPaymentDone = ref(true)
  const postNeglectedPaymentFunc = async (consultationId: number) => {
    try {
      postNeglectedPaymentDone.value = false
      const response = await postNeglectedPayment(consultationId)
      return response
    } finally {
      postNeglectedPaymentDone.value = true
    }
  }
  return {
    postNeglectedPaymentDone,
    postNeglectedPaymentFunc
  }
}
