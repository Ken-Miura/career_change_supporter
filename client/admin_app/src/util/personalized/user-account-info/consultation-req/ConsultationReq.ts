export type ConsultationReq = {
  /* eslint-disable camelcase */
  consultation_req_id: number,
  user_account_id: number,
  consultant_id: number,
  first_candidate_date_time: string, // RFC 3339形式の文字列
  second_candidate_date_time: string, // RFC 3339形式の文字列
  third_candidate_date_time: string, // RFC 3339形式の文字列
  latest_candidate_date_time: string, // RFC 3339形式の文字列
  charge_id: string,
  fee_per_hour_in_yen: number,
  platform_fee_rate_in_percentage: string,
  credit_facilities_expired_at: string, // RFC 3339形式の文字列
  /* eslint-enable camelcase */
}
