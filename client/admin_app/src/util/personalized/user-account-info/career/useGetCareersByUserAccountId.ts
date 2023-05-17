import { ref } from 'vue'
import { getCareersByUserAccountId } from './GetCareersByUserAccountId'

export function useGetCareersByUserAccountId () {
  const getCareersByUserAccountIdDone = ref(true)
  const getCareersByUserAccountIdFunc = async (userAccountId: string) => {
    try {
      getCareersByUserAccountIdDone.value = false
      const response = await getCareersByUserAccountId(userAccountId)
      return response
    } finally {
      getCareersByUserAccountIdDone.value = true
    }
  }
  return {
    getCareersByUserAccountIdDone,
    getCareersByUserAccountIdFunc
  }
}
