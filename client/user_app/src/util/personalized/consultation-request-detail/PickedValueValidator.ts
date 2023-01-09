export function checkIfPickedValueIsInValidRange (pickedValue: string): boolean {
  if (pickedValue === '1' || pickedValue === '2' || pickedValue === '3') {
    return true
  }
  return false
}
