export type ReceiptOfConsultation = {
    /* eslint-disable camelcase */
    consultation_id: number,
    user_account_id: number,
    consultant_id: number,
    meeting_at: string, // RFC 3339形式の文字列,
    fee_per_hour_in_yen: number,
    platform_fee_rate_in_percentage: string,
    transfer_fee_in_yen: number,
    reward: number,
    sender_name: string,
    bank_code: string,
    branch_code: string,
    account_type: string,
    account_number: string,
    account_holder_name: string,
    withdrawal_confirmed_by: string,
    created_at: string, // RFC 3339形式の文字列
    /* eslint-enable camelcase */
}
