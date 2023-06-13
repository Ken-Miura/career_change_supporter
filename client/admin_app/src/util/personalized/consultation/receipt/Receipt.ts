export type Receipt = {
  /* eslint-disable camelcase */
  receipt_id: number,
  consultation_id: number,
  charge_id: string,
  fee_per_hour_in_yen: number,
  platform_fee_rate_in_percentage: string,
  settled_at: string, // RFC 3339形式の文字列
  /* eslint-enable camelcase */
}
