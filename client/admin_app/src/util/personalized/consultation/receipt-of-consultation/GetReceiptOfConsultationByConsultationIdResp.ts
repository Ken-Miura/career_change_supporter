import { ReceiptOfConsultation } from '../../ReceiptOfConsultation'

export class GetReceiptOfConsultationByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly item: ReceiptOfConsultation | null) {}

  public static create (item: ReceiptOfConsultation | null): GetReceiptOfConsultationByConsultationIdResp {
    return new GetReceiptOfConsultationByConsultationIdResp(item)
  }

  public getReceiptOfConsultation (): ReceiptOfConsultation | null {
    return this.item
  }
}
