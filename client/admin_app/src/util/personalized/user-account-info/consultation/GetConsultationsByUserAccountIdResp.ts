import { ConsultationsResult } from './ConsultationsResult'

export class GetConsultationsByUserAccountIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly consultationsResult: ConsultationsResult) {}

  public static create (consultationsResult: ConsultationsResult): GetConsultationsByUserAccountIdResp {
    return new GetConsultationsByUserAccountIdResp(consultationsResult)
  }

  public getConsultationsResult (): ConsultationsResult {
    return this.consultationsResult
  }
}
