import { RefundedPayment } from './RefundedPayment'

export class GetRefundedPaymentsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly items: RefundedPayment[]) {}
  public static create (items: RefundedPayment[]): GetRefundedPaymentsResp {
    return new GetRefundedPaymentsResp(items)
  }

  public getItems (): RefundedPayment[] {
    return this.items
  }
}
