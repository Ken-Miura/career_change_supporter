export function getCurrentYear (): number {
  const d = new Date()
  return d.getFullYear()
}

export function getCurrentMonth (): number {
  const d = new Date()
  return d.getMonth() + 1
}
