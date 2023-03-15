import { ref } from 'vue'

// eslint-disable-next-line
export function useRequestConsultationDone () {
  const requestConsultationDone = ref(true)
  const beginRequestConsultation = () => {
    requestConsultationDone.value = false
  }
  const finishRequestConsultation = () => {
    requestConsultationDone.value = true
  }
  const disabled = ref(false)
  const disableBtn = () => {
    disabled.value = true
  }
  const enableBtn = () => {
    disabled.value = false
  }
  return {
    requestConsultationDone,
    beginRequestConsultation,
    finishRequestConsultation,
    disabled,
    disableBtn,
    enableBtn
  }
}
