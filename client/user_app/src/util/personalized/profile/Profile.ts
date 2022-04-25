import { Identity } from './Identity'
import { CareerDescription } from './CareerDescription'

export type Profile = {
    /* eslint-disable camelcase */
    email_address: string,
    identity: Identity | null,
    career_descriptions: CareerDescription[],
    fee_per_hour_in_yen: number | null,
    /* eslint-enable camelcase */
}
