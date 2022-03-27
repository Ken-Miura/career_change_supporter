import { CreateIdentityRequestDetail } from './CreateIdentityRequestDetail'

export class GetCreateIdentityRequestDetailResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly detail: CreateIdentityRequestDetail) {}
  public static create (detail: CreateIdentityRequestDetail): GetCreateIdentityRequestDetailResp {
    return new GetCreateIdentityRequestDetailResp(detail)
  }

  public getDetail (): CreateIdentityRequestDetail {
    return this.detail
  }
}
