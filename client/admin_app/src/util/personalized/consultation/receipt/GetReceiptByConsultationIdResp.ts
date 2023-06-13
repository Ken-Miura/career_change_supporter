import { ReceiptResult } from './ReceiptResult'

export class GetReceiptByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly receiptResult: ReceiptResult) {}

  public static create (receiptResult: ReceiptResult): GetReceiptByConsultationIdResp {
    return new GetReceiptByConsultationIdResp(receiptResult)
  }

  public getReceiptResult (): ReceiptResult {
    return this.receiptResult
  }
}
