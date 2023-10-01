import { ref } from 'vue'
import { postConsultantRating } from './PostConsultantRating'

export function usePostConsultantRating () {
  const postConsultantRatingDone = ref(true)
  const postConsultantRatingFunc = async (consultationId: number, rating: number) => {
    try {
      postConsultantRatingDone.value = false
      const response = await postConsultantRating(consultationId, rating)
      return response
    } finally {
      postConsultantRatingDone.value = true
    }
  }
  return {
    postConsultantRatingDone,
    postConsultantRatingFunc
  }
}
