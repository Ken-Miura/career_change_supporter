import { ref } from 'vue'
import { postReceiptOfConsultation } from './PostReceiptOfConsultation'

export function usePostReceiptOfConsultation () {
  const postReceiptOfConsultationDone = ref(true)
  const postReceiptOfConsultationFunc = async (consultationId: number) => {
    try {
      postReceiptOfConsultationDone.value = false
      const response = await postReceiptOfConsultation(consultationId)
      return response
    } finally {
      postReceiptOfConsultationDone.value = true
    }
  }
  return {
    postReceiptOfConsultationDone,
    postReceiptOfConsultationFunc
  }
}
