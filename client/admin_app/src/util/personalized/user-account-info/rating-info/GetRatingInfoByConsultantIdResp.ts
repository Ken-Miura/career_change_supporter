import { RatingInfoResult } from './RatingInfoResult'

export class GetRatingInfoByConsultantIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly ratingInfoResult: RatingInfoResult) {}

  public static create (ratingInfoResult: RatingInfoResult): GetRatingInfoByConsultantIdResp {
    return new GetRatingInfoByConsultantIdResp(ratingInfoResult)
  }

  public getRatingInfoResult (): RatingInfoResult {
    return this.ratingInfoResult
  }
}
