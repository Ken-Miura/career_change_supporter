export type ConsultantCareerDescription = {
  /* eslint-disable camelcase */
  company_name: string,
  profession: string | null,
  office: string | null,
  /* eslint-enable camelcase */
}

export type ConsultantDescription = {
  /* eslint-disable camelcase */
  consultant_id: number,
  fee_per_hour_in_yen: number,
  rating: string | null,
  num_of_rated: number,
  careers: ConsultantCareerDescription[],
  /* eslint-enable camelcase */
}

export type ConsultantsSearchResult = {
  total: number,
  consultants: ConsultantDescription[]
}
