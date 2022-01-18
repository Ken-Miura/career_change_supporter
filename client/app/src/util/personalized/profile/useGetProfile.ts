import { getProfile } from '@/util/personalized/profile/GetProfile'
import { ref } from 'vue'

// eslint-disable-next-line
export function useGetProfile () {
  const getProfileDone = ref(false)
  const getProfileFunc = async () => {
    try {
      const response = await getProfile()
      getProfileDone.value = true
      return response
    } catch (e) {
      getProfileDone.value = true
      throw e
    }
  }
  return {
    getProfileDone,
    getProfileFunc
  }
}
