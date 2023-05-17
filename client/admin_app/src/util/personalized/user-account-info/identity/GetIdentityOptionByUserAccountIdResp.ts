import { IdentityResult } from './IdentityResult'

export class GetIdentityOptionByUserAccountIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly identityResult: IdentityResult) {}

  public static create (identityResult: IdentityResult): GetIdentityOptionByUserAccountIdResp {
    return new GetIdentityOptionByUserAccountIdResp(identityResult)
  }

  public getIdentityResult (): IdentityResult {
    return this.identityResult
  }
}
