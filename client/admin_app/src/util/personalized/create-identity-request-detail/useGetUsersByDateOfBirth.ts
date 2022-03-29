import { ref } from 'vue'
import { getUsersByDateOfBirth } from './GetUsersByDateOfBirth'

export function useGetUsersByDateOfBirth () {
  const waitingGetUsersByDateOfBirthDone = ref(false)
  const getUsersByDateOfBirthFunc = async (year: number, month: number, day: number) => {
    try {
      waitingGetUsersByDateOfBirthDone.value = true
      const response = await getUsersByDateOfBirth(year, month, day)
      return response
    } finally {
      waitingGetUsersByDateOfBirthDone.value = false
    }
  }
  return {
    waitingGetUsersByDateOfBirthDone,
    getUsersByDateOfBirthFunc
  }
}
