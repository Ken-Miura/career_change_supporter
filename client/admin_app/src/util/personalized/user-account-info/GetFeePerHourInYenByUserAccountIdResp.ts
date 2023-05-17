import { FeePerHourInYenResult } from './FeePerHourInYenResult'

export class GetFeePerHourInYenByUserAccountIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly feePerHourInYenResult: FeePerHourInYenResult) {}

  public static create (feePerHourInYenResult: FeePerHourInYenResult): GetFeePerHourInYenByUserAccountIdResp {
    return new GetFeePerHourInYenByUserAccountIdResp(feePerHourInYenResult)
  }

  public getFeePerHourInYenResult (): FeePerHourInYenResult {
    return this.feePerHourInYenResult
  }
}
