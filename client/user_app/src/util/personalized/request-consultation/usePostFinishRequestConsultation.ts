import { ref } from 'vue'
import { FinishRequestConsultation } from './FinishRequestConsultation'
import { postFinishRequestConsultation } from './PostFinishRequestConsultation'

// eslint-disable-next-line
export function usePostFinishRequestConsultation () {
  const postFinishRequestConsultationDone = ref(true)
  const postFinishRequestConsultationFunc = async (finishRequestConsultation: FinishRequestConsultation) => {
    try {
      postFinishRequestConsultationDone.value = false
      const response = await postFinishRequestConsultation(finishRequestConsultation)
      return response
    } finally {
      postFinishRequestConsultationDone.value = true
    }
  }
  return {
    postFinishRequestConsultationDone,
    postFinishRequestConsultationFunc
  }
}
