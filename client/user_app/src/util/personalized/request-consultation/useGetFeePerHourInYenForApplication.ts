import { ref } from 'vue'
import { getFeePerHourInYenForApplication } from './GetFeePerHourInYenForApplication'

// eslint-disable-next-line
export function useGetFeePerHourInYenForApplication () {
  const getFeePerHourInYenForApplicationDone = ref(true)
  const getFeePerHourInYenForApplicationFunc = async (consultantId: string) => {
    try {
      getFeePerHourInYenForApplicationDone.value = false
      const response = await getFeePerHourInYenForApplication(consultantId)
      return response
    } finally {
      getFeePerHourInYenForApplicationDone.value = true
    }
  }
  return {
    getFeePerHourInYenForApplicationDone,
    getFeePerHourInYenForApplicationFunc
  }
}
