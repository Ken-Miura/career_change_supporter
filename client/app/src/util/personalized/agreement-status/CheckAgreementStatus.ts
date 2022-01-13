import { ApiError, ApiErrorResp } from '../../ApiError'
import { CheckAgreementStatusResp } from './CheckAgreementStatusResp'

/**
 * 利用規約に同意済かどうか確認する。<br>
 * 利用規約に同意済かどうか確認する前に下記の処理を行う。<br>
 * <ul>
 *   <li>ログインセッションが存在するかどうか確認する（ログインセッションが存在しない場合、Code.UNAUTHORIZEDを返す）</li>
 *   <li>ログインセッションが存在する場合、ログインセッションを延長する</li>
 * </ul>
 * POST /api/agreementとPOST /api/logoutを除くpersonalized以下にあるAPI呼び出しは、そのAPIの処理の前にこのAPI呼び出しでの処理と同じ処理を行う。<br>
 * そのため、何かpersonalizedでのAPI呼び出しをする際は、このAPIを明示的に呼び出す必要はない。<br>
 * サーバからデータを取得する必要がないログイン後のページを表示する際での利用を想定<br>
 * @returns Promise<CheckAgreementStatusResp | ApiErrorResp>
 */
export async function checkAgreementStatus (): Promise<CheckAgreementStatusResp | ApiErrorResp> {
  const response = await fetch('/api/agreement-status', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return CheckAgreementStatusResp.create()
}
