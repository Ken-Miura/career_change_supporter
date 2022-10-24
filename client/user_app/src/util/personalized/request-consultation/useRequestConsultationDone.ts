import { ref } from 'vue'

// eslint-disable-next-line
export function useRequestConsultationDone () {
  const requestConsultationDone = ref(true)
  return {
    requestConsultationDone
  }
}
