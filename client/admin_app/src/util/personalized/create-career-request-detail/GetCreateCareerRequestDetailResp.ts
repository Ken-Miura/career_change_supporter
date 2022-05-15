import { CreateCareerRequestDetail } from './CreateCareerRequestDetail'

export class GetCreateCareerRequestDetailResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly detail: CreateCareerRequestDetail) {}
  public static create (detail: CreateCareerRequestDetail): GetCreateCareerRequestDetailResp {
    return new GetCreateCareerRequestDetailResp(detail)
  }

  public getDetail (): CreateCareerRequestDetail {
    return this.detail
  }
}
