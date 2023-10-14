import { AwaitingWithdrawal } from './AwaitingWithdrawal'

export class GetAwaitingWithdrawalByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly item: AwaitingWithdrawal | null) {}

  public static create (item: AwaitingWithdrawal | null): GetAwaitingWithdrawalByConsultationIdResp {
    return new GetAwaitingWithdrawalByConsultationIdResp(item)
  }

  public getAwaitingWithdrawal (): AwaitingWithdrawal | null {
    return this.item
  }
}
