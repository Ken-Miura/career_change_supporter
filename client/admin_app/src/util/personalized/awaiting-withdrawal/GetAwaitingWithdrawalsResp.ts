import { AwaitingWithdrawal } from './AwaitingWithdrawal'

export class GetAwaitingWithdrawalsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly items: AwaitingWithdrawal[]) {}
  public static create (items: AwaitingWithdrawal[]): GetAwaitingWithdrawalsResp {
    return new GetAwaitingWithdrawalsResp(items)
  }

  public getItems (): AwaitingWithdrawal[] {
    return this.items
  }
}
