import { computed, reactive } from 'vue'

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

  const allCandidatesAreNotEmpty = computed(() => {
    return candidates.firstCandidateYearInJst !== '' &&
      candidates.firstCandidateMonthInJst !== '' &&
      candidates.firstCandidateDayInJst !== '' &&
      candidates.firstCandidateHourInJst !== '' &&
      candidates.secondCandidateYearInJst !== '' &&
      candidates.secondCandidateMonthInJst !== '' &&
      candidates.secondCandidateDayInJst !== '' &&
      candidates.secondCandidateHourInJst !== '' &&
      candidates.thirdCandidateYearInJst !== '' &&
      candidates.thirdCandidateMonthInJst !== '' &&
      candidates.thirdCandidateDayInJst !== '' &&
      candidates.thirdCandidateHourInJst !== ''
  })

  const checkIfCandidatesAreSame = (year1: string, month1: string, day1: string, hour1: string, year2: string, month2: string, day2: string, hour2: string) => {
    return year1 === year2 &&
      month1 === month2 &&
      day1 === day2 &&
      hour1 === hour2
  }
  const sameCandidatesExist = computed(() => {
    return checkIfCandidatesAreSame(candidates.firstCandidateYearInJst, candidates.firstCandidateMonthInJst, candidates.firstCandidateDayInJst, candidates.firstCandidateHourInJst, candidates.secondCandidateYearInJst, candidates.secondCandidateMonthInJst, candidates.secondCandidateDayInJst, candidates.secondCandidateHourInJst) ||
      checkIfCandidatesAreSame(candidates.firstCandidateYearInJst, candidates.firstCandidateMonthInJst, candidates.firstCandidateDayInJst, candidates.firstCandidateHourInJst, candidates.thirdCandidateYearInJst, candidates.thirdCandidateMonthInJst, candidates.thirdCandidateDayInJst, candidates.thirdCandidateHourInJst) ||
      checkIfCandidatesAreSame(candidates.thirdCandidateYearInJst, candidates.thirdCandidateMonthInJst, candidates.thirdCandidateDayInJst, candidates.thirdCandidateHourInJst, candidates.secondCandidateYearInJst, candidates.secondCandidateMonthInJst, candidates.secondCandidateDayInJst, candidates.secondCandidateHourInJst)
  })

  return {
    candidates,
    allCandidatesAreNotEmpty,
    sameCandidatesExist
  }
}
