export function getCurrentYear (): number {
  const d = new Date()
  return d.getFullYear()
}

export function getCurrentMonth (): number {
  const d = new Date()
  return d.getMonth() + 1
}

export function getCurrentDate (): Date {
  return new Date()
}
