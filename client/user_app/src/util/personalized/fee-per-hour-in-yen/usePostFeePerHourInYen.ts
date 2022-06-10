import { ref } from 'vue'
import { postFeePerHourInYen } from './PostFeePerHourInYen'

export function usePostFeePerHourInYen () {
  const postFeePerHourInYenDone = ref(true)
  const postFeePerHourInYenFunc = async (feePerHourInYen: number) => {
    try {
      postFeePerHourInYenDone.value = false
      const response = await postFeePerHourInYen(feePerHourInYen)
      return response
    } finally {
      postFeePerHourInYenDone.value = true
    }
  }
  return {
    postFeePerHourInYenDone,
    postFeePerHourInYenFunc
  }
}
