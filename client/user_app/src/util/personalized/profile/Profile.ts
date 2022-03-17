import { Identity } from './Identity'
import { Career } from './Career'

export type Profile = {
    /* eslint-disable camelcase */
    email_address: string,
    identity: Identity | null,
    careers: Career[],
    fee_per_hour_in_yen: number | null,
    /* eslint-enable camelcase */
}
