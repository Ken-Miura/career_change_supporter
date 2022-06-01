import { ref } from 'vue'
import { getCareer } from './GetCareer'

export function useGetCareer () {
  const getCareerDone = ref(true)
  const getCareerFunc = async (careerId: number) => {
    try {
      getCareerDone.value = false
      const response = await getCareer(careerId)
      return response
    } finally {
      getCareerDone.value = true
    }
  }
  return {
    getCareerDone,
    getCareerFunc
  }
}
