import { CreateCareerRequestItem } from './CreateCareerRequestItem'

export class GetCreateCareerRequestsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly items: CreateCareerRequestItem[]) {}
  public static create (items: CreateCareerRequestItem[]): GetCreateCareerRequestsResp {
    return new GetCreateCareerRequestsResp(items)
  }

  public getItems (): CreateCareerRequestItem[] {
    return this.items
  }
}
