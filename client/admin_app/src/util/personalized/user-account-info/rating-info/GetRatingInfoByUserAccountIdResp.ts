import { RatingInfoResult } from './RatingInfoResult'

export class GetRatingInfoByUserAccountIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly ratingInfoResult: RatingInfoResult) {}

  public static create (ratingInfoResult: RatingInfoResult): GetRatingInfoByUserAccountIdResp {
    return new GetRatingInfoByUserAccountIdResp(ratingInfoResult)
  }

  public getRatingInfoResult (): RatingInfoResult {
    return this.ratingInfoResult
  }
}
