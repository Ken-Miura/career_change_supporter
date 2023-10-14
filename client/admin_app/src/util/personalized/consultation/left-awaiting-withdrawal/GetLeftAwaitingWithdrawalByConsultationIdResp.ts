import { LeftAwaitingWithdrawal } from '../../LeftAwaitingWithdrawal'

export class GetLeftAwaitingWithdrawalByConsultationIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly item: LeftAwaitingWithdrawal | null) {}

  public static create (item: LeftAwaitingWithdrawal | null): GetLeftAwaitingWithdrawalByConsultationIdResp {
    return new GetLeftAwaitingWithdrawalByConsultationIdResp(item)
  }

  public getLeftAwaitingWithdrawal (): LeftAwaitingWithdrawal | null {
    return this.item
  }
}
