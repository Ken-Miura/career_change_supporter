import { getCurrentDate } from './CurrentDateTime'
import { getMaxDurationBeforeConsultationInDays, getMinDurationBeforeConsultationInDays } from './DurationBeforeConsultation'

export function checkIfCandidateIsInValidRange (year: string, month: string, day: string, hour: string): boolean {
  const zeroIndexedMonth = parseInt(month) - 1
  const candidate = new Date(parseInt(year), zeroIndexedMonth, parseInt(day), parseInt(hour))

  const currentDate = getCurrentDate()

  const min = new Date(currentDate.getTime())
  min.setHours(min.getHours() + (getMinDurationBeforeConsultationInDays() * 24))

  const max = new Date(currentDate.getTime())
  max.setHours(max.getHours() + (getMaxDurationBeforeConsultationInDays() * 24))

  return min.getTime() <= candidate.getTime() && candidate.getTime() <= max.getTime()
}
