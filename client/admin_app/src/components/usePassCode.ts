import { ref } from 'vue'

// eslint-disable-next-line
export function usePassCode () {
  const passCode = ref('')
  const setPassCode = (pc: string) => {
    passCode.value = pc
  }
  return {
    passCode,
    setPassCode
  }
}
