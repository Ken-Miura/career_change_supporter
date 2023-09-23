export type ConsultationReq = {
  /* eslint-disable camelcase */
  consultation_req_id: number,
  user_account_id: number,
  consultant_id: number,
  first_candidate_date_time: string, // RFC 3339形式の文字列
  second_candidate_date_time: string, // RFC 3339形式の文字列
  third_candidate_date_time: string, // RFC 3339形式の文字列
  latest_candidate_date_time: string, // RFC 3339形式の文字列
  fee_per_hour_in_yen: number,
  /* eslint-enable camelcase */
}
