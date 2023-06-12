export type ConsultantRating = {
  /* eslint-disable camelcase */
  consultant_rating_id: number,
  consultation_id: number,
  rating: number | null,
  rated_at: string | null, // RFC 3339形式の文字列
  /* eslint-enable camelcase */
}
