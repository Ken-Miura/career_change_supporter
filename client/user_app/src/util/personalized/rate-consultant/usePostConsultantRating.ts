import { ref } from 'vue'
import { postConsultantRating } from './PostConsultantRating'

export function usePostConsultantRating () {
  const postConsultantRatingDone = ref(true)
  const postConsultantRatingFunc = async (consultantRatingId: number, rating: number) => {
    try {
      postConsultantRatingDone.value = false
      const response = await postConsultantRating(consultantRatingId, rating)
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
