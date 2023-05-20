import { ConsultationReqsResult } from './ConsultationReqsResult'

export class GetConsultationReqsByUserAccountIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly consultationReqsResult: ConsultationReqsResult) {}

  public static create (consultationReqsResult: ConsultationReqsResult): GetConsultationReqsByUserAccountIdResp {
    return new GetConsultationReqsByUserAccountIdResp(consultationReqsResult)
  }

  public getConsultationReqsResult (): ConsultationReqsResult {
    return this.consultationReqsResult
  }
}
