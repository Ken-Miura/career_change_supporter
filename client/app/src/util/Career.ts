import { Ymd } from './Ymd'

export type Career = {
    /* eslint-disable camelcase */
    company_name: string,
    department_name: string | null,
    office: string | null,
    career_start_date: Ymd,
    career_end_date: Ymd | null,
    contract_type: 'regular' | 'contract' | 'other',
    profession: string | null,
    annual_income_in_man_yen: number | null,
    is_manager: boolean,
    position_name: string | null,
    is_new_graduate: boolean,
    note: string | null,
    /* eslint-enable camelcase */
}
