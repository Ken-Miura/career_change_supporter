import { IdentityCreationRejectionRecordResult } from './IdentityCreationRejectionRecordResult'

export class GetIdentityCreationRejectionRecordResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly rejectionRecordResult: IdentityCreationRejectionRecordResult) {}

  public static create (rejectionRecordResult: IdentityCreationRejectionRecordResult): GetIdentityCreationRejectionRecordResp {
    return new GetIdentityCreationRejectionRecordResp(rejectionRecordResult)
  }

  public getIdentityCreationRejectionRecordResult (): IdentityCreationRejectionRecordResult {
    return this.rejectionRecordResult
  }
}
