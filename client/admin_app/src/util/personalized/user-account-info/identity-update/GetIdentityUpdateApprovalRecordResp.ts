import { IdentityUpdateApprovalRecordResult } from './IdentityUpdateApprovalRecordResult'

export class GetIdentityUpdateApprovalRecordResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly approvalRecordResult: IdentityUpdateApprovalRecordResult) {}

  public static create (approvalRecordResult: IdentityUpdateApprovalRecordResult): GetIdentityUpdateApprovalRecordResp {
    return new GetIdentityUpdateApprovalRecordResp(approvalRecordResult)
  }

  public getIdentityUpdateApprovalRecordResult (): IdentityUpdateApprovalRecordResult {
    return this.approvalRecordResult
  }
}
