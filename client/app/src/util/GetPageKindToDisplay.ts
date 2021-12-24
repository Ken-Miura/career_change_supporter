import { checkAgreementStatus } from './agreement-status/CheckAgreementStatus'
import { CheckAgreementStatusResp } from './agreement-status/CheckAgreementStatusResp'
import { ApiErrorResp } from './ApiError'
import { Code } from './Error'
import { refresh } from './refresh/Refresh'

export type PageKind = 'login' | 'term-of-use' | 'personalized-page'

export async function getPageKindToDisplay (): Promise<PageKind> {
  try {
    const result = await refresh()
    if (!result) {
      return 'login'
    }
    // セッションが存在するので、利用規約の確認
    const agreementStatus = await checkAgreementStatus()
    if (agreementStatus instanceof CheckAgreementStatusResp) {
      // セッションが存在し、利用規約に同意済のため、ログイン後のページを表示可能
      return 'personalized-page'
    } else if (agreementStatus instanceof ApiErrorResp) {
      const code = agreementStatus.getApiError().getCode()
      if (code === Code.UNAUTHORIZED) {
        return 'login'
      } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
        return 'term-of-use'
      } else {
        throw new Error(`unexpected result: ${agreementStatus}`)
      }
    }
  } catch (e) {
    // 例外が発生した場合は無視し、ログイン画面に遷移
    // ログイン画面に遷移させ、ログイン時の動作でエラーを確認してもらう
  }
  return 'login'
}
