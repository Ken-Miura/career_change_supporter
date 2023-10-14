export type NeglectedPayment = {
    /* eslint-disable camelcase */
    consultation_id: number,
    user_account_id: number,
    consultant_id: number,
    meeting_at: string, // RFC 3339形式の文字列,
    fee_per_hour_in_yen: number,
    neglect_confirmed_by: string,
    created_at: string, // RFC 3339形式の文字列
    /* eslint-enable camelcase */
}
