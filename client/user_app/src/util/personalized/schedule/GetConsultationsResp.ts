import { ConsultationsResult } from './ConsultationsResult'

export class GetConsultationsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly consultationsResult: ConsultationsResult) {}

  public static create (consultationsResult: ConsultationsResult): GetConsultationsResp {
    return new GetConsultationsResp(consultationsResult)
  }

  public getConsultationsResult (): ConsultationsResult {
    return this.consultationsResult
  }
}
