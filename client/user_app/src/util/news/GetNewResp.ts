import { NewsResult } from './NewsResult'

export class GetNewResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly newsResult: NewsResult) {}

  public static create (newsResult: NewsResult): GetNewResp {
    return new GetNewResp(newsResult)
  }

  public getNewsResult (): NewsResult {
    return this.newsResult
  }
}
