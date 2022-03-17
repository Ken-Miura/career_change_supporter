import { getRewards } from '@/util/personalized/reward/GetRewards'
import { ref } from 'vue'

// eslint-disable-next-line
export function useGetRewards () {
  const getRewardsDone = ref(false)
  const getRewardsFunc = async () => {
    try {
      const response = await getRewards()
      return response
    } finally {
      getRewardsDone.value = true
    }
  }
  return {
    getRewardsDone,
    getRewardsFunc
  }
}
