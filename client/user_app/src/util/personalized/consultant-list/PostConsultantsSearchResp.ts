import { ConsultantsSearchResult } from './ConsultantsSearchResult'

export class PostConsultantsSearchResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly consultantsSearchResult: ConsultantsSearchResult) {}

  public static create (consultantsSearchResult: ConsultantsSearchResult): PostConsultantsSearchResp {
    return new PostConsultantsSearchResp(consultantsSearchResult)
  }

  public getConsultantsSearchResult (): ConsultantsSearchResult {
    return this.consultantsSearchResult
  }
}
