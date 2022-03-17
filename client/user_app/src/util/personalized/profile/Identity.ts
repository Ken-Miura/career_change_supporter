import { Ymd } from '../../Ymd'

export type Identity = {
    /* eslint-disable camelcase */
    last_name: string,
    first_name: string,
    last_name_furigana: string,
    first_name_furigana: string,
    date_of_birth: Ymd,
    prefecture: string,
    city: string,
    address_line1: string,
    address_line2: string | null,
    telephone_number: string,
    /* eslint-enable camelcase */
}
