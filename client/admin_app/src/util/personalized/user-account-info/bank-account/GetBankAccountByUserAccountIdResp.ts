import { BankAccount } from './BankAccount'

export class GetBankAccountByUserAccountIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly bankAccount: BankAccount | null) {}

  public static create (bankAccount: BankAccount | null): GetBankAccountByUserAccountIdResp {
    return new GetBankAccountByUserAccountIdResp(bankAccount)
  }

  public getBankAccount (): BankAccount | null {
    return this.bankAccount
  }
}
