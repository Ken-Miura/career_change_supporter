import { ref } from 'vue'
import { getRatingInfoByUserAccountId } from './GetRatingInfoByUserAccountId'

export function useGetRatingInfoByUserAccountId () {
  const getRatingInfoByUserAccountIdDone = ref(true)
  const getRatingInfoByUserAccountIdFunc = async (userAccountId: string) => {
    try {
      getRatingInfoByUserAccountIdDone.value = false
      const response = await getRatingInfoByUserAccountId(userAccountId)
      return response
    } finally {
      getRatingInfoByUserAccountIdDone.value = true
    }
  }
  return {
    getRatingInfoByUserAccountIdDone,
    getRatingInfoByUserAccountIdFunc
  }
}
