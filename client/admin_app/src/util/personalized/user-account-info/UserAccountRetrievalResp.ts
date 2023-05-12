import { UserAccountRetrievalResult } from './UserAccountRetrievalResult'

export class UserAccountRetrievalResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly result: UserAccountRetrievalResult) {}

  public static create (result: UserAccountRetrievalResult): UserAccountRetrievalResp {
    return new UserAccountRetrievalResp(result)
  }

  public getResult (): UserAccountRetrievalResult {
    return this.result
  }
}
