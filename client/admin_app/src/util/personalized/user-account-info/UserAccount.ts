export type UserAccount = {
  /* eslint-disable camelcase */
  user_account_id: number,
  email_address: string,
  last_login_time: string | null, // RFC 3339形式の文字列
  created_at: string, // RFC 3339形式の文字列
  mfa_enabled_at: string | null, // RFC 3339形式の文字列
  disabled_at: string | null, // RFC 3339形式の文字列
  /* eslint-enable camelcase */
}
