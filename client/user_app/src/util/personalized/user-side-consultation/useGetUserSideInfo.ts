import { ref } from 'vue'
import { getUserSideInfo } from './GetUserSideInfo'

// eslint-disable-next-line
export function useGetUserSideInfo () {
  const getUserSideInfoDone = ref(true)
  const getUserSideInfoFunc = async (consultationId: string) => {
    try {
      getUserSideInfoDone.value = false
      const response = await getUserSideInfo(consultationId)
      return response
    } finally {
      getUserSideInfoDone.value = true
    }
  }
  return {
    getUserSideInfoDone,
    getUserSideInfoFunc
  }
}
