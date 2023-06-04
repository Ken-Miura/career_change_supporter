import { UserAccountRetrievalResult } from '../UserAccountRetrievalResult'

export class PostEnableUserAccountReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly result: UserAccountRetrievalResult) {}

  public static create (result: UserAccountRetrievalResult): PostEnableUserAccountReqResp {
    return new PostEnableUserAccountReqResp(result)
  }

  public getUserAccountRetrievalResult (): UserAccountRetrievalResult {
    return this.result
  }
}
