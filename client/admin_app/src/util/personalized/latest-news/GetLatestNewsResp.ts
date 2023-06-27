import { LatestNewsResult } from './LatestNewsResult'

export class GetLatestNewsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly latestNewsResult: LatestNewsResult) {}

  public static create (latestNewsResult: LatestNewsResult): GetLatestNewsResp {
    return new GetLatestNewsResp(latestNewsResult)
  }

  public getLatestNewsResult (): LatestNewsResult {
    return this.latestNewsResult
  }
}
