export type StoppedSettlement = {
  /* eslint-disable camelcase */
  stopped_settlement_id: number,
  consultation_id: number,
  charge_id: string,
  fee_per_hour_in_yen: number,
  platform_fee_rate_in_percentage: string,
  credit_facilities_expired_at: string, // RFC 3339形式の文字列
  stopped_at: string, // RFC 3339形式の文字列
  /* eslint-enable camelcase */
}
