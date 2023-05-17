import { ref } from 'vue'
import { getFeePerHourInYenByUserAccountId } from '../fee-per-hour-in-yen/GetFeePerHourInYenByUserAccountId'

export function useGetFeePerHourInYenByUserAccountId () {
  const getFeePerHourInYenByUserAccountIdDone = ref(true)
  const getFeePerHourInYenByUserAccountIdFunc = async (userAccountId: string) => {
    try {
      getFeePerHourInYenByUserAccountIdDone.value = false
      const response = await getFeePerHourInYenByUserAccountId(userAccountId)
      return response
    } finally {
      getFeePerHourInYenByUserAccountIdDone.value = true
    }
  }
  return {
    getFeePerHourInYenByUserAccountIdDone,
    getFeePerHourInYenByUserAccountIdFunc
  }
}
