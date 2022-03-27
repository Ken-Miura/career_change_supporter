import { Ymd } from "@/util/Ymd";

export type User = {
    /* eslint-disable camelcase */
    user_account_id: number,
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
