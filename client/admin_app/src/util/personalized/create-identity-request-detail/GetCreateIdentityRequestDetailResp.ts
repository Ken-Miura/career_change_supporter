import { CreateIdentityReqDetail } from './CreateIdentityReqDetail'

export class GetCreateIdentityRequestDetailResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly detail: CreateIdentityReqDetail) {}
  public static create (detail: CreateIdentityReqDetail): GetCreateIdentityRequestDetailResp {
    return new GetCreateIdentityRequestDetailResp(detail)
  }

  public getDetail (): CreateIdentityReqDetail {
    return this.detail
  }
}
