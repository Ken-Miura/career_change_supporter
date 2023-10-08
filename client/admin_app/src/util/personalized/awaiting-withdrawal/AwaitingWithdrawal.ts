export type AwaitingWithdrawal = {
    /* eslint-disable camelcase */
    consultation_id: number,
    user_account_id: number,
    consultant_id: number,
    meeting_at: string, // RFC 3339形式の文字列
    fee_per_hour_in_yen: number,
    sender_name: string,
    payment_confirmed_by: string,
    created_at: string, // RFC 3339形式の文字列
    bank_code: string | null,
    branch_code: string | null,
    account_type: string | null,
    account_number: string | null,
    account_holder_name: string | null,
    /* eslint-enable camelcase */
}
