import { ConsultationReqsResult } from './ConsultationReqsResult'

export class GetConsultationReqsByConsultantIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly consultationReqsResult: ConsultationReqsResult) {}

  public static create (consultationReqsResult: ConsultationReqsResult): GetConsultationReqsByConsultantIdResp {
    return new GetConsultationReqsByConsultantIdResp(consultationReqsResult)
  }

  public getConsultationReqsResult (): ConsultationReqsResult {
    return this.consultationReqsResult
  }
}
