export const START_YEAR = 1960

export function createYearList (startYear: number, currentYear: number): string[] {
  const list = [] as string[]
  list.push('')
  const end = startYear - 1
  for (let i = currentYear; i > end; i--) {
    list.push(i.toString())
  }
  return list
}
