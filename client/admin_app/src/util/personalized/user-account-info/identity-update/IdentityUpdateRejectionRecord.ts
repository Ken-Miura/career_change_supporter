export type IdentityUpdateRejectionRecord = {
  /* eslint-disable camelcase */
  rjd_upd_identity_id: number,
  user_account_id: number,
  last_name: string,
  first_name: string,
  last_name_furigana: string,
  first_name_furigana: string,
  date_of_birth: string, // 2023-05-27 のような形式の文字列
  prefecture: string,
  city: string,
  address_line1: string,
  address_line2: string | null,
  telephone_number: string,
  reason: string,
  rejected_at: string, // RFC 3339形式の文字列
  rejected_by: string,
  /* eslint-enable camelcase */
}
