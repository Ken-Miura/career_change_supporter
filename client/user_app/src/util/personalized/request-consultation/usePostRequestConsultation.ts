import { ref } from 'vue'
import { ConsultationRequest } from './ConsultationRequest'
import { postRequestConsultation } from './PostRequestConsultation'

// eslint-disable-next-line
export function usePostRequestConsultation () {
  const postRequestConsultationDone = ref(true)
  const postRequestConsultationFunc = async (consultationRequest: ConsultationRequest) => {
    try {
      postRequestConsultationDone.value = false
      const response = await postRequestConsultation(consultationRequest)
      return response
    } finally {
      postRequestConsultationDone.value = true
    }
  }
  return {
    postRequestConsultationDone,
    postRequestConsultationFunc
  }
}
