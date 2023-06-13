import { StoppedSettlementResult } from './StoppedSettlementResult'

export class GetStoppedSettlementByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly stoppedSettlementResult: StoppedSettlementResult) {}

  public static create (stoppedSettlementResult: StoppedSettlementResult): GetStoppedSettlementByConsultationIdResp {
    return new GetStoppedSettlementByConsultationIdResp(stoppedSettlementResult)
  }

  public getStoppedSettlementResult (): StoppedSettlementResult {
    return this.stoppedSettlementResult
  }
}
