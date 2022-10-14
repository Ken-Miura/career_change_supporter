import { BankAccount } from '../BankAccount'
import { Transfer } from './Transfer'

export type Rewards = {
    /* eslint-disable camelcase */
    bank_account: BankAccount | null,
    rewards_of_the_month: number | null,
    rewards_of_the_year: number | null,
    latest_two_transfers: Transfer[],
    /* eslint-enable camelcase */
}
