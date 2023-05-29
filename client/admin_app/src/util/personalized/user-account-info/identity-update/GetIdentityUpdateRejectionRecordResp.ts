import { IdentityUpdateRejectionRecordResult } from './IdentityUpdateRejectionRecordResult'

export class GetIdentityUpdateRejectionRecordResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly rejectionRecordResult: IdentityUpdateRejectionRecordResult) {}

  public static create (rejectionRecordResult: IdentityUpdateRejectionRecordResult): GetIdentityUpdateRejectionRecordResp {
    return new GetIdentityUpdateRejectionRecordResp(rejectionRecordResult)
  }

  public getIdentityUpdateRejectionRecordResult (): IdentityUpdateRejectionRecordResult {
    return this.rejectionRecordResult
  }
}
