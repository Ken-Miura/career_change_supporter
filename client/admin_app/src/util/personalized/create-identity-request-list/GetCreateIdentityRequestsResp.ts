import { CreateIdentityRequestItem } from './CreateIdentityRequestItem'

export class GetCreateIdentityRequests {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
    private constructor (private readonly items: CreateIdentityRequestItem[]) {}
  public static create (items: CreateIdentityRequestItem[]): GetCreateIdentityRequests {
    return new GetCreateIdentityRequests(items)
  }

  public getItems (): CreateIdentityRequestItem[] {
    return this.items
  }
}
