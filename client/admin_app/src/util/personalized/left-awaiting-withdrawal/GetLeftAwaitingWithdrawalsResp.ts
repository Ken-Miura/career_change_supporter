import { LeftAwaitingWithdrawal } from './LeftAwaitingWithdrawal'

export class GetLeftAwaitingWithdrawalsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly items: LeftAwaitingWithdrawal[]) {}
  public static create (items: LeftAwaitingWithdrawal[]): GetLeftAwaitingWithdrawalsResp {
    return new GetLeftAwaitingWithdrawalsResp(items)
  }

  public getItems (): LeftAwaitingWithdrawal[] {
    return this.items
  }
}
