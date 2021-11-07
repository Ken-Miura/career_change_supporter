/**
 * セッションの有効期限を延長する。
 * 有効期限を延長するためには、セッションの有効期限内に呼び出す必要がある。
 * @returns 成功の場合、trueを返す。失敗の場合、falseを返す。
 */
export async function refresh (): Promise<boolean> {
  const response = await fetch('/api/refresh', {
    method: 'GET'
  })
  if (!response.ok) {
    return false
  }
  return true
}
