import { NeglectedPayment } from './NeglectedPayment'

export class GetNeglectedPaymentsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly items: NeglectedPayment[]) {}
  public static create (items: NeglectedPayment[]): GetNeglectedPaymentsResp {
    return new GetNeglectedPaymentsResp(items)
  }

  public getItems (): NeglectedPayment[] {
    return this.items
  }
}
