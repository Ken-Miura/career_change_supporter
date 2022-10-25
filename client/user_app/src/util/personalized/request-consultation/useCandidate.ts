import { reactive } from 'vue'

// eslint-disable-next-line
export function useCandidate () {
  const candidates = reactive({
    firstCandidateYearInJst: '',
    firstCandidateMonthInJst: '',
    firstCandidateDayInJst: '',
    firstCandidateHourInJst: '',
    secondCandidateYearInJst: '',
    secondCandidateMonthInJst: '',
    secondCandidateDayInJst: '',
    secondCandidateHourInJst: '',
    thirdCandidateYearInJst: '',
    thirdCandidateMonthInJst: '',
    thirdCandidateDayInJst: '',
    thirdCandidateHourInJst: ''
  })
  return {
    candidates
  }
}
