export function getCurrentMonth (): number {
  const d = new Date()
  return d.getMonth() + 1
}

export function createMonthList (currentMonth: number): string[] {
  let nextMonth
  if (currentMonth === 12) {
    nextMonth = 1
  } else {
    nextMonth = currentMonth + 1
  }
  const list = []
  list.push('')
  list.push(currentMonth.toString())
  list.push(nextMonth.toString())
  return list
}
