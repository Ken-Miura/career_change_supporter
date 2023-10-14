import { ReceiptOfConsultation } from '../ReceiptOfConsultation'

export class GetReceiptsOfConsultationResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly items: ReceiptOfConsultation[]) {}
  public static create (items: ReceiptOfConsultation[]): GetReceiptsOfConsultationResp {
    return new GetReceiptsOfConsultationResp(items)
  }

  public getItems (): ReceiptOfConsultation[] {
    return this.items
  }
}
