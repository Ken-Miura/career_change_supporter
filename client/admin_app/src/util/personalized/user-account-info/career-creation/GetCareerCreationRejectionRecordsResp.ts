import { CareerCreationRejectionRecordsResult } from './CareerCreationRejectionRecordsResult'

export class GetCareerCreationRejectionRecordsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly rejectionRecordsResult: CareerCreationRejectionRecordsResult) {}

  public static create (rejectionRecordsResult: CareerCreationRejectionRecordsResult): GetCareerCreationRejectionRecordsResp {
    return new GetCareerCreationRejectionRecordsResp(rejectionRecordsResult)
  }

  public getCareerCreationRejectionRecordsResult (): CareerCreationRejectionRecordsResult {
    return this.rejectionRecordsResult
  }
}
