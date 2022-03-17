import { getProfile } from '@/util/personalized/profile/GetProfile'
import { ref } from 'vue'

// eslint-disable-next-line
export function useGetProfile () {
  const getProfileDone = ref(false)
  const getProfileFunc = async () => {
    try {
      const response = await getProfile()
      return response
    } finally {
      getProfileDone.value = true
    }
  }
  return {
    getProfileDone,
    getProfileFunc
  }
}
