export type RefundedPayment = {
    /* eslint-disable camelcase */
    consultation_id: number,
    user_account_id: number,
    consultant_id: number,
    meeting_at: string, // RFC 3339形式の文字列,
    fee_per_hour_in_yen: number,
    transfer_fee_in_yen: number,
    sender_name: string,
    reason: string,
    refund_confirmed_by: string,
    created_at: string, // RFC 3339形式の文字列
    /* eslint-enable camelcase */
}
