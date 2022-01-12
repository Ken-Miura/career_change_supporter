import { getRewards } from '@/util/personalized/reward/GetRewards'
import { ref } from 'vue'

// eslint-disable-next-line
export function useGetRewards () {
  const getRewardsDone = ref(false)
  const getRewardsFunc = async () => {
    const response = await getRewards()
    getRewardsDone.value = true
    return response
  }
  return {
    getRewardsDone,
    getRewardsFunc
  }
}
