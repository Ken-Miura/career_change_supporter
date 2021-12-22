import { Identity } from './Identity'
import { Career } from './Career'
import { BankAccount } from './BankAccount'
import { Transfer } from './Transfer'

export type Profile = {
    /* eslint-disable camelcase */
    email_address: string,
    identity: Identity | null,
    careers: Career[],
    fee_per_hour_in_yen: number | null,
    bank_account: BankAccount | null,
    profit: number | null,
    latest_two_transfers: Transfer[],
    /* eslint-enable camelcase */
}
