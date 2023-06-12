import { SettlementResult } from './SettlementResult'

export class GetSettlementByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly settlementResult: SettlementResult) {}

  public static create (settlementResult: SettlementResult): GetSettlementByConsultationIdResp {
    return new GetSettlementByConsultationIdResp(settlementResult)
  }

  public getSettlementResult (): SettlementResult {
    return this.settlementResult
  }
}
