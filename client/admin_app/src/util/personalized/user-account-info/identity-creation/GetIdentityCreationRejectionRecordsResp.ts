import { IdentityCreationRejectionRecordsResult } from './IdentityCreationRejectionRecordsResult'

export class GetIdentityCreationRejectionRecordsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly rejectionRecordsResult: IdentityCreationRejectionRecordsResult) {}

  public static create (rejectionRecordsResult: IdentityCreationRejectionRecordsResult): GetIdentityCreationRejectionRecordsResp {
    return new GetIdentityCreationRejectionRecordsResp(rejectionRecordsResult)
  }

  public getIdentityCreationRejectionRecordsResult (): IdentityCreationRejectionRecordsResult {
    return this.rejectionRecordsResult
  }
}
