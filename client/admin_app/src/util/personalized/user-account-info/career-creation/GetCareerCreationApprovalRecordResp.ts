import { CareerCreationApprovalRecordResult } from './CareerCreationApprovalRecordResult'

export class GetCareerCreationApprovalRecordResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly approvalRecordResult: CareerCreationApprovalRecordResult) {}

  public static create (approvalRecordResult: CareerCreationApprovalRecordResult): GetCareerCreationApprovalRecordResp {
    return new GetCareerCreationApprovalRecordResp(approvalRecordResult)
  }

  public getCareerCreationApprovalRecordResult (): CareerCreationApprovalRecordResult {
    return this.approvalRecordResult
  }
}
