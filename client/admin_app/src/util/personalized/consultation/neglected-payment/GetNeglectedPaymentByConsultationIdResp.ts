import { NeglectedPayment } from '../../NeglectedPayment'

export class GetNeglectedPaymentByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly item: NeglectedPayment | null) {}

  public static create (item: NeglectedPayment | null): GetNeglectedPaymentByConsultationIdResp {
    return new GetNeglectedPaymentByConsultationIdResp(item)
  }

  public getNeglectedPayment (): NeglectedPayment | null {
    return this.item
  }
}
