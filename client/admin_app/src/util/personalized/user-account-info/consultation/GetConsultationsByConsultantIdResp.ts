import { ConsultationsResult } from './ConsultationsResult'

export class GetConsultationsByConsultantIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly consultationsResult: ConsultationsResult) {}

  public static create (consultationsResult: ConsultationsResult): GetConsultationsByConsultantIdResp {
    return new GetConsultationsByConsultantIdResp(consultationsResult)
  }

  public getConsultationsResult (): ConsultationsResult {
    return this.consultationsResult
  }
}
