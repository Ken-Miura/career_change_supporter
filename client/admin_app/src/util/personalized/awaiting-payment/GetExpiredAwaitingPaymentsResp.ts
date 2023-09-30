import { AwaitingPayment } from './AwaitingPayment'

export class GetExpiredAwaitingPaymentsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly items: AwaitingPayment[]) {}
  public static create (items: AwaitingPayment[]): GetExpiredAwaitingPaymentsResp {
    return new GetExpiredAwaitingPaymentsResp(items)
  }

  public getItems (): AwaitingPayment[] {
    return this.items
  }
}
