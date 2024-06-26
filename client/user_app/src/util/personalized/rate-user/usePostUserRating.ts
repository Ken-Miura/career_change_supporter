import { ref } from 'vue'
import { postUserRating } from './PostUserRating'

export function usePostUserRating () {
  const postUserRatingDone = ref(true)
  const postUserRatingFunc = async (consultationId: number, rating: number) => {
    try {
      postUserRatingDone.value = false
      const response = await postUserRating(consultationId, rating)
      return response
    } finally {
      postUserRatingDone.value = true
    }
  }
  return {
    postUserRatingDone,
    postUserRatingFunc
  }
}
