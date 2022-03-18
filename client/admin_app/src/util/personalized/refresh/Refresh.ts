import { ApiError, ApiErrorResp } from '../../ApiError'
import { RefreshResp } from './RefreshResp'

/**
 * 下記の処理を順に行う。
 * - ログインセッションが存在するかどうか確認する（ログインセッションが存在しない場合、Code.UNAUTHORIZEDを返す）
 * - ログインセッションが存在する場合、ログインセッションを延長する
 * - 利用規約に同意済かどうか確認する（利用規約に同意済でない場合、Code.NOT_TERMS_OF_USE_AGREED_YETを返す）
 *
 * POST /api/agreementとPOST /api/logoutを除くpersonalized以下にあるAPI呼び出しは、そのAPI自体の処理の前にこのAPI呼び出しと同等の処理を行う。<br>
 * そのため、何かpersonalizedでのAPI呼び出しをする際は、このAPIを明示的に呼び出す必要はない。<br>
 * ログイン後のページで、サーバからデータを取得する必要がないものを表示する際での利用を想定<br>
 * @returns Promise<RefreshResp | ApiErrorResp>
 */
export async function refresh (): Promise<RefreshResp | ApiErrorResp> {
  const response = await fetch('/api/refresh', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return RefreshResp.create()
}
