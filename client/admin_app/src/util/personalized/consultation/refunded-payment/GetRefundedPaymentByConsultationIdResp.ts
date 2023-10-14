import { RefundedPayment } from '../../RefundedPayment'

export class GetRefundedPaymentByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly item: RefundedPayment | null) {}

  public static create (item: RefundedPayment | null): GetRefundedPaymentByConsultationIdResp {
    return new GetRefundedPaymentByConsultationIdResp(item)
  }

  public getRefundedPayment (): RefundedPayment | null {
    return this.item
  }
}
