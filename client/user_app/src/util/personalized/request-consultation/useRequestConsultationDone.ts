import { ref } from 'vue'

// eslint-disable-next-line
export function useRequestConsultationDone () {
  const requestConsultationDone = ref(true)
  const startRequestConsultation = () => {
    requestConsultationDone.value = false
  }
  const finishRequestConsultation = () => {
    requestConsultationDone.value = true
  }
  return {
    requestConsultationDone,
    startRequestConsultation,
    finishRequestConsultation
  }
}
