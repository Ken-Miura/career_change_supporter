import { CreateIdentityRequestItem } from './CreateIdentityRequestItem'

export class GetCreateIdentityRequestsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly items: CreateIdentityRequestItem[]) {}
  public static create (items: CreateIdentityRequestItem[]): GetCreateIdentityRequestsResp {
    return new GetCreateIdentityRequestsResp(items)
  }

  public getItems (): CreateIdentityRequestItem[] {
    return this.items
  }
}
