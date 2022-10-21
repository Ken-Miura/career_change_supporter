export function getCurrentYear (): number {
  const d = new Date()
  return d.getFullYear()
}

export function createYearList (currentYear: number): string[] {
  const list = []
  list.push('')
  list.push(currentYear.toString())
  list.push((currentYear + 1).toString())
  return list
}
