import { ref } from 'vue'
import { getCareer } from './GetCareer'

export function useGetCareer () {
  const getCareerDone = ref(false)
  const getCareerFunc = async (careerId: number) => {
    try {
      getCareerDone.value = true
      const response = await getCareer(careerId)
      return response
    } finally {
      getCareerDone.value = false
    }
  }
  return {
    getCareerDone,
    getCareerFunc
  }
}
