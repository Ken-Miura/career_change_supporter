import { ref } from 'vue'
import { getAwaitingRatings } from './GetAwaitingRatings'

// eslint-disable-next-line
export function useGetAwaitingRatings () {
  const getAwaitingRatingsDone = ref(true)
  const getAwaitingRatingsFunc = async () => {
    try {
      getAwaitingRatingsDone.value = false
      const response = await getAwaitingRatings()
      return response
    } finally {
      getAwaitingRatingsDone.value = true
    }
  }
  return {
    getAwaitingRatingsDone,
    getAwaitingRatingsFunc
  }
}
