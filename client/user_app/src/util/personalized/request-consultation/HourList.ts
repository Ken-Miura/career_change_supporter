export const FIRST_START_HOUR_OF_CONSULTATION = 7
export const LAST_START_HOUR_OF_CONSULTATION = 23

export function createHourList (): string[] {
  const list = []
  list.push('')
  for (let i = FIRST_START_HOUR_OF_CONSULTATION; i <= LAST_START_HOUR_OF_CONSULTATION; i++) {
    list.push(i.toString())
  }
  return list
}
