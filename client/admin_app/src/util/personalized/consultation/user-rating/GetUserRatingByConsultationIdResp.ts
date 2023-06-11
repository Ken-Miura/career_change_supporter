import { UserRatingResult } from './UserRatingResult'

export class GetUserRatingByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly userRatingResult: UserRatingResult) {}

  public static create (userRatingResult: UserRatingResult): GetUserRatingByConsultationIdResp {
    return new GetUserRatingByConsultationIdResp(userRatingResult)
  }

  public getUserRatingResult (): UserRatingResult {
    return this.userRatingResult
  }
}
