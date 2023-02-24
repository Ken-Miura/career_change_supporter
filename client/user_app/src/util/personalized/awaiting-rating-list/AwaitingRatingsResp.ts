import { AwaitingRatings } from './AwaitingRatings'

export class AwaitingRatingsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly awaitingRatings: AwaitingRatings) {}

  public static create (awaitingRatings: AwaitingRatings): AwaitingRatingsResp {
    return new AwaitingRatingsResp(awaitingRatings)
  }

  public getAwaitingRatings (): AwaitingRatings {
    return this.awaitingRatings
  }
}
