import { ConsultationRequestsResult } from './ConsultationRequestsResult'

export class GetConsultationRequestsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly consultationRequestsResult: ConsultationRequestsResult) {}

  public static create (consultationRequestsResult: ConsultationRequestsResult): GetConsultationRequestsResp {
    return new GetConsultationRequestsResp(consultationRequestsResult)
  }

  public getConsultationRequestsResult (): ConsultationRequestsResult {
    return this.consultationRequestsResult
  }
}
