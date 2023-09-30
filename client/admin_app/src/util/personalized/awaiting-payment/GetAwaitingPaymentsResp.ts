import { AwaitingPayment } from './AwaitingPayment'

export class GetAwaitingPaymentsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly items: AwaitingPayment[]) {}
  public static create (items: AwaitingPayment[]): GetAwaitingPaymentsResp {
    return new GetAwaitingPaymentsResp(items)
  }

  public getItems (): AwaitingPayment[] {
    return this.items
  }
}
