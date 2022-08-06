export type AnnualInComeInManYenParam = {
  /* eslint-disable camelcase */
  equal_or_more: number | null,
  equal_or_less: number | null,
  /* eslint-enable camelcase */
}

export type CareerParam = {
    /* eslint-disable camelcase */
    company_name: string | null,
    department_name: string | null,
    office: string | null,
    years_of_service: string | null,
    employed: boolean | null,
    contract_type: string | null,
    profession: string | null,
    annual_income_in_man_yen: AnnualInComeInManYenParam,
    is_manager: boolean | null,
    position_name: string | null,
    is_new_graduate: boolean | null,
    note: string | null
    /* eslint-enable camelcase */
}

export type FeePerHourInYenParam = {
    /* eslint-disable camelcase */
    equal_or_more: number | null,
    equal_or_less: number | null,
    /* eslint-enable camelcase */
}

export type SortParam = {
  key: string,
  order: string
}

export type ConsultantSearchParam = {
  /* eslint-disable camelcase */
  career_param: CareerParam,
  fee_per_hour_in_yen_param: FeePerHourInYenParam,
  sort_param: SortParam | null,
  from: number,
  size: number,
  /* eslint-enable camelcase */
}
