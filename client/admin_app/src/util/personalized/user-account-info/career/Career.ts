export type Career = {
  /* eslint-disable camelcase */
  career_id: number,
  user_account_id: number,
  company_name: string,
  department_name: string | null,
  office: string | null,
  career_start_date: string,
  career_end_date: string | null,
  contract_type: 'regular' | 'contract' | 'other',
  profession: string | null,
  annual_income_in_man_yen: number | null,
  is_manager: boolean,
  position_name: string | null,
  is_new_graduate: boolean,
  note: string | null,
  /* eslint-enable camelcase */
}
