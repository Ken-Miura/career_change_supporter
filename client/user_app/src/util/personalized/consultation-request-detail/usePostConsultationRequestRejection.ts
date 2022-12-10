import { ref } from 'vue'
import { ConsultationRequestRejectionParam } from './ConsultationRequestRejectionParam'
import { postConsultationRequestRejection } from './PostConsultationRequestRejection'

export function usePostConsultationRequestRejection () {
  const postConsultationRequestRejectionDone = ref(true)
  const postConsultationRequestRejectionFunc = async (param: ConsultationRequestRejectionParam) => {
    try {
      postConsultationRequestRejectionDone.value = false
      const response = await postConsultationRequestRejection(param)
      return response
    } finally {
      postConsultationRequestRejectionDone.value = true
    }
  }
  return {
    postConsultationRequestRejectionDone,
    postConsultationRequestRejectionFunc
  }
}
