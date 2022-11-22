export type ConsultationRequestDescription = {
  /* eslint-disable camelcase */
  consultation_req_id: number,
  user_account_id: number,
  /* eslint-enable camelcase */
}

export type ConsultationRequestsResult = {
  /* eslint-disable camelcase */
  consultation_requests: ConsultationRequestDescription[]
  /* eslint-enable camelcase */
}
