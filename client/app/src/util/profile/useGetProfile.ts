import { getProfile } from '@/util/profile/GetProfile'
import { ref } from 'vue'

// eslint-disable-next-line
export function useGetProfile () {
  const getProfileDone = ref(false)
  const getProfileFunc = async () => {
    const response = await getProfile()
    getProfileDone.value = true
    return response
  }
  return {
    getProfileDone,
    getProfileFunc
  }
}
