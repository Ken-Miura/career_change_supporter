import { UpdateIdentityRequestDetail } from './UpdateIdentityRequestDetail'

export class GetUpdateIdentityRequestDetailResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly detail: UpdateIdentityRequestDetail) {}
  public static create (detail: UpdateIdentityRequestDetail): GetUpdateIdentityRequestDetailResp {
    return new GetUpdateIdentityRequestDetailResp(detail)
  }

  public getDetail (): UpdateIdentityRequestDetail {
    return this.detail
  }
}
