import { CareerCreationRejectionRecordResult } from './CareerCreationRejectionRecordResult'

export class GetCareerCreationRejectionRecordResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly rejectionRecordResult: CareerCreationRejectionRecordResult) {}

  public static create (rejectionRecordResult: CareerCreationRejectionRecordResult): GetCareerCreationRejectionRecordResp {
    return new GetCareerCreationRejectionRecordResp(rejectionRecordResult)
  }

  public getCareerCreationRejectionRecordResult (): CareerCreationRejectionRecordResult {
    return this.rejectionRecordResult
  }
}
