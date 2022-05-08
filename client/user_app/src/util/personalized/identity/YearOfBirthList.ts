export const START_YEAR = 1950
export const MIN_AGE = 18

export function createYearOfBirthList (startYear: number, currentYear: number, minAge: number): string[] {
  const list = [] as string[]
  const endYear = currentYear - minAge
  for (let i = endYear; i > (startYear - 1); i--) {
    list.push(i.toString())
  }
  return list
}
