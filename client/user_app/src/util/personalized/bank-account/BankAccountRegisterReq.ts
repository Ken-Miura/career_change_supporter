import { BankAccount } from '../BankAccount'

export type BankAccountRegisterReq = {
  /* eslint-disable camelcase */
  bank_account: BankAccount,
  non_profit_objective: boolean
  /* eslint-enable camelcase */
}
