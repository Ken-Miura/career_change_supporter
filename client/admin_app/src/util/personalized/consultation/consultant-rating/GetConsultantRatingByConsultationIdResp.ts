import { ConsultantRatingResult } from './ConsultantRatingResult'

export class GetConsultantRatingByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly consultantRatingResult: ConsultantRatingResult) {}

  public static create (consultantRatingResult: ConsultantRatingResult): GetConsultantRatingByConsultationIdResp {
    return new GetConsultantRatingByConsultationIdResp(consultantRatingResult)
  }

  public getConsultantRatingResult (): ConsultantRatingResult {
    return this.consultantRatingResult
  }
}
