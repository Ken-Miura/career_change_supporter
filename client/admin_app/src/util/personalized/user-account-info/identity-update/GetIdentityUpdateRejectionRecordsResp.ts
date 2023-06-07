import { IdentityUpdateRejectionRecordsResult } from './IdentityUpdateRejectionRecordsResult'

export class GetIdentityUpdateRejectionRecordsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly rejectionRecordsResult: IdentityUpdateRejectionRecordsResult) {}

  public static create (rejectionRecordsResult: IdentityUpdateRejectionRecordsResult): GetIdentityUpdateRejectionRecordsResp {
    return new GetIdentityUpdateRejectionRecordsResp(rejectionRecordsResult)
  }

  public getIdentityUpdateRejectionRecordsResult (): IdentityUpdateRejectionRecordsResult {
    return this.rejectionRecordsResult
  }
}
