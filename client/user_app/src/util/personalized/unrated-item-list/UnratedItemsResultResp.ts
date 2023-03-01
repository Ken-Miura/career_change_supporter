import { UnratedItemsResult } from './UnratedItemsResult'

export class UnratedItemsResultResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly unratedItemsResult: UnratedItemsResult) {}

  public static create (unratedItemsResult: UnratedItemsResult): UnratedItemsResultResp {
    return new UnratedItemsResultResp(unratedItemsResult)
  }

  public getUnratedItemsResult (): UnratedItemsResult {
    return this.unratedItemsResult
  }
}
