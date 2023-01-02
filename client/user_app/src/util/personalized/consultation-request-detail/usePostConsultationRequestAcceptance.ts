import { ref } from 'vue'
import { ConsultationRequestAcceptanceParam } from './ConsultationRequestAcceptanceParam'
import { postConsultationRequestAcceptance } from './PostConsultationRequestAcceptance'

export function usePostConsultationRequestAcceptance () {
  const postConsultationRequestAcceptanceDone = ref(true)
  const postConsultationRequestAcceptanceFunc = async (param: ConsultationRequestAcceptanceParam) => {
    try {
      postConsultationRequestAcceptanceDone.value = false
      const response = await postConsultationRequestAcceptance(param)
      return response
    } finally {
      postConsultationRequestAcceptanceDone.value = true
    }
  }
  return {
    postConsultationRequestAcceptanceDone,
    postConsultationRequestAcceptanceFunc
  }
}
