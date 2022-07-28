import { ref } from 'vue'
import { ConsultantSearchParam } from '../ConsultantSearchParam'
import { postConsultantsSearch } from './PostConsultantsSearch'

// eslint-disable-next-line
export function usePostConsultantsSearch () {
  const postConsultantsSearchDone = ref(true)
  const postConsultantsSearchFunc = async (consultantSearchParam: ConsultantSearchParam) => {
    try {
      postConsultantsSearchDone.value = false
      const response = await postConsultantsSearch(consultantSearchParam)
      return response
    } finally {
      postConsultantsSearchDone.value = true
    }
  }
  return {
    postConsultantsSearchDone,
    postConsultantsSearchFunc
  }
}
