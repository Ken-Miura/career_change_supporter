import { UserAccountRetrievalResult } from '../UserAccountRetrievalResult'

export class PostDisableUserAccountReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly result: UserAccountRetrievalResult) {}

  public static create (result: UserAccountRetrievalResult): PostDisableUserAccountReqResp {
    return new PostDisableUserAccountReqResp(result)
  }

  public getUserAccountRetrievalResult (): UserAccountRetrievalResult {
    return this.result
  }
}
