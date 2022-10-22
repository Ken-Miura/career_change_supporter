export function getCurrentYear (): number {
  const d = new Date()
  return d.getFullYear()
}

export function createYearList (currentMonth: number, currentYear: number): string[] {
  const list = []
  list.push('')
  list.push(currentYear.toString())
  if (currentMonth === 12) {
    list.push((currentYear + 1).toString())
  }
  return list
}
