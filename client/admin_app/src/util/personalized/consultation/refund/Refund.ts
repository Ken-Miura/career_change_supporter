export type Refund = {
  /* eslint-disable camelcase */
  refund_id: number,
  consultation_id: number,
  charge_id: string,
  fee_per_hour_in_yen: number,
  platform_fee_rate_in_percentage: string,
  settled_at: string, // RFC 3339形式の文字列
  refunded_at: string, // RFC 3339形式の文字列
  /* eslint-enable camelcase */
}
