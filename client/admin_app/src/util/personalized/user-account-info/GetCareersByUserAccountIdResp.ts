import { CareersResult } from './CareersResult'

export class GetCareersByUserAccountIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly careersResult: CareersResult) {}

  public static create (careersResult: CareersResult): GetCareersByUserAccountIdResp {
    return new GetCareersByUserAccountIdResp(careersResult)
  }

  public getCareersResult (): CareersResult {
    return this.careersResult
  }
}
