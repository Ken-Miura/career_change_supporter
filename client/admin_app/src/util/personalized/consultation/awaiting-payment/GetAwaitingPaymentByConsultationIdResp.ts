import { AwaitingPayment } from './AwaitingPayment'

export class GetAwaitingPaymentByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly item: AwaitingPayment | null) {}

  public static create (item: AwaitingPayment | null): GetAwaitingPaymentByConsultationIdResp {
    return new GetAwaitingPaymentByConsultationIdResp(item)
  }

  public getAwaitingPayment (): AwaitingPayment | null {
    return this.item
  }
}
