import { UserAccountRetrievalResult } from '../UserAccountRetrievalResult'

export class PostDisableMfaReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly result: UserAccountRetrievalResult) {}

  public static create (result: UserAccountRetrievalResult): PostDisableMfaReqResp {
    return new PostDisableMfaReqResp(result)
  }

  public getUserAccountRetrievalResult (): UserAccountRetrievalResult {
    return this.result
  }
}
