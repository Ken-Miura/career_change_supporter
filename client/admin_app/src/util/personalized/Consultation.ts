export type Consultation = {
  /* eslint-disable camelcase */
  consultation_id: number,
  user_account_id: number,
  consultant_id: number,
  meeting_at: string, // RFC 3339形式の文字列
  room_name: string,
  user_account_entered_at: string | null, // RFC 3339形式の文字列
  consultant_entered_at: string | null, // RFC 3339形式の文字列
  /* eslint-enable camelcase */
}
