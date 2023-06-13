import { RefundResult } from './RefundResult'

export class GetRefundByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly refundResult: RefundResult) {}

  public static create (refundResult: RefundResult): GetRefundByConsultationIdResp {
    return new GetRefundByConsultationIdResp(refundResult)
  }

  public getRefundResult (): RefundResult {
    return this.refundResult
  }
}
