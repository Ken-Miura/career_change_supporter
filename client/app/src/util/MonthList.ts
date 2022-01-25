export type Month = '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '10' | '11' | '12'

export function createMonthList (): Month[] {
  const list = [] as Month[]
  list.push('1')
  list.push('2')
  list.push('3')
  list.push('4')
  list.push('5')
  list.push('6')
  list.push('7')
  list.push('8')
  list.push('9')
  list.push('10')
  list.push('11')
  list.push('12')
  return list
}
