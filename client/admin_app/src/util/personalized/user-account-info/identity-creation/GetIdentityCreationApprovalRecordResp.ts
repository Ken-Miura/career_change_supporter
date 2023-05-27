import { IdentityCreationApprovalRecordResult } from './IdentityCreationApprovalRecordResult'

export class GetIdentityCreationApprovalRecordResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly approvalRecordResult: IdentityCreationApprovalRecordResult) {}

  public static create (approvalRecordResult: IdentityCreationApprovalRecordResult): GetIdentityCreationApprovalRecordResp {
    return new GetIdentityCreationApprovalRecordResp(approvalRecordResult)
  }

  public getIdentityCreationApprovalRecordResult (): IdentityCreationApprovalRecordResult {
    return this.approvalRecordResult
  }
}
