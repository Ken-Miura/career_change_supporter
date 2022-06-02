import { ref } from 'vue'
import { deleteCareer } from './DeleteCareer'

export function useDeleteCareer () {
  const deleteCareerDone = ref(true)
  const deleteCareerFunc = async (careerId: number) => {
    try {
      deleteCareerDone.value = false
      const response = await deleteCareer(careerId)
      return response
    } finally {
      deleteCareerDone.value = true
    }
  }
  return {
    deleteCareerDone,
    deleteCareerFunc
  }
}
