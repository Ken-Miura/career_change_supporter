import { IdentityUpdateApprovalRecordsResult } from './IdentityUpdateApprovalRecordsResult'

export class GetIdentityUpdateApprovalRecordsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly approvalRecordsResult: IdentityUpdateApprovalRecordsResult) {}

  public static create (approvalRecordsResult: IdentityUpdateApprovalRecordsResult): GetIdentityUpdateApprovalRecordsResp {
    return new GetIdentityUpdateApprovalRecordsResp(approvalRecordsResult)
  }

  public getIdentityUpdateApprovalRecordsResult (): IdentityUpdateApprovalRecordsResult {
    return this.approvalRecordsResult
  }
}
