import { ConsultationResult } from './ConsultationResult'

export class GetConsultationByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly ConsultationResult: ConsultationResult) {}

  public static create (ConsultationResult: ConsultationResult): GetConsultationByConsultationIdResp {
    return new GetConsultationByConsultationIdResp(ConsultationResult)
  }

  public getConsultationResult (): ConsultationResult {
    return this.ConsultationResult
  }
}
