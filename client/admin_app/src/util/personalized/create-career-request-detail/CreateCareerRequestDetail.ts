import { Ymd } from '@/util/Ymd'

export type CreateCareerRequestDetail = {
    /* eslint-disable camelcase */
    user_account_id: number,
    company_name: string,
    department_name: string | null,
    office: string | null,
    career_start_date: Ymd,
    career_end_date: Ymd | null,
    contract_type: string,
    profession: string | null,
    annual_income_in_man_yen: number | null,
    is_manager: boolean,
    position_name: string | null,
    is_new_graduate: boolean,
    note: string | null,
    image1_file_name_without_ext: string,
    image2_file_name_without_ext: string | null,
    /* eslint-enable camelcase */
}
