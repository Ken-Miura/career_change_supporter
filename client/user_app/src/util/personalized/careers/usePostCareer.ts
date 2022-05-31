import { postCareer } from '@/util/personalized/careers/PostCareer'
import { ref } from 'vue'
import { Career } from '../Career'

// eslint-disable-next-line
export function usePostCareer () {
  const waitingRequestDone = ref(false)
  const postCareerFunc = async (career: Career, image1: File, image2: File | null) => {
    try {
      waitingRequestDone.value = true
      const response = await postCareer(career, image1, image2)
      return response
    } finally {
      waitingRequestDone.value = false
    }
  }
  return {
    waitingRequestDone,
    postCareerFunc
  }
}
