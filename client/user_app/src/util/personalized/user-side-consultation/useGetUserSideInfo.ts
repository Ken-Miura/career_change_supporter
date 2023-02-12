import { ref } from 'vue'
import { getUserSideInfo } from './GetUserSideInfo'

// eslint-disable-next-line
export function useGetUserSideInfo () {
  const getUserSideInfoDone = ref(true)
  const getUserSideInfoFunc = async (consultationId: string, audioTestDone: boolean) => {
    try {
      getUserSideInfoDone.value = false
      const response = await getUserSideInfo(consultationId, audioTestDone)
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
