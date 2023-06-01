export type CareerCreationRejectionRecord = {
  /* eslint-disable camelcase */
  rjd_cre_career_req_id: number,
  user_account_id: number,
  company_name: string,
  department_name: string | null,
  office: string | null,
  career_start_date: string, // 2023-05-27 のような形式の文字列
  career_end_date: string | null, // 2023-05-27 のような形式の文字列
  contract_type: string, // 'regular' or 'contract' or 'other'
  profession: string | null,
  annual_income_in_man_yen: number | null,
  is_manager: boolean,
  position_name: string | null,
  is_new_graduate: boolean,
  note: string | null,
  reason: string,
  rejected_at: string, // RFC 3339形式の文字列
  rejected_by: string,
  /* eslint-enable camelcase */
}
