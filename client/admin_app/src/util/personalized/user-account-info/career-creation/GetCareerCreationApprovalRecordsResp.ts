import { CareerCreationApprovalRecordsResult } from './CareerCreationApprovalRecordsResult'

export class GetCareerCreationApprovalRecordsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly approvalRecordsResult: CareerCreationApprovalRecordsResult) {}

  public static create (approvalRecordsResult: CareerCreationApprovalRecordsResult): GetCareerCreationApprovalRecordsResp {
    return new GetCareerCreationApprovalRecordsResp(approvalRecordsResult)
  }

  public getCareerCreationApprovalRecordsResult (): CareerCreationApprovalRecordsResult {
    return this.approvalRecordsResult
  }
}
