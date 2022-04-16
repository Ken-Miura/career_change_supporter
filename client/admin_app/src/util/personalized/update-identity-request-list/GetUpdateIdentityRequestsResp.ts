import { UpdateIdentityRequestItem } from './UpdateIdentityRequestItem'

export class GetUpdateIdentityRequestsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly items: UpdateIdentityRequestItem[]) {}
  public static create (items: UpdateIdentityRequestItem[]): GetUpdateIdentityRequestsResp {
    return new GetUpdateIdentityRequestsResp(items)
  }

  public getItems (): UpdateIdentityRequestItem[] {
    return this.items
  }
}
