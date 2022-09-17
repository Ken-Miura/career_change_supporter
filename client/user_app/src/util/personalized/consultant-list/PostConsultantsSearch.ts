import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { ConsultantSearchParam } from '../ConsultantSearchParam'
import { ConsultantCareerDescription, ConsultantDescription, ConsultantsSearchResult } from './ConsultantsSearchResult'
import { ConsultantsSearchResult as RawConsultantsSearchResult } from './raw-response/ConsultantsSearchResult'
import { PostConsultantsSearchResp } from './PostConsultantsSearchResp'

export async function postConsultantsSearch (consultantSearchParam: ConsultantSearchParam): Promise<PostConsultantsSearchResp | ApiErrorResp> {
  const response = await fetch('/api/consultants-search', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(consultantSearchParam)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const rawResult = await response.json() as RawConsultantsSearchResult
  const result = convertResult(rawResult)
  return PostConsultantsSearchResp.create(result)
}

// ConsultantDescriptionのcareersの要素内に一意に識別できる値は含まれていない。
// v-forで回す際に一意な識別子が必要になるので、返ってきたレスポンスを画面に表示する前に一意な識別子を生成して含めておく
function convertResult (rawResult: RawConsultantsSearchResult): ConsultantsSearchResult {
  const consultants = [] as ConsultantDescription[]
  for (const rawConsultant of rawResult.consultants) {
    const careerDescriptions = [] as ConsultantCareerDescription[]
    for (let i = 0; i < rawConsultant.careers.length; i++) {
      careerDescriptions.push({
        consultant_career_id: i,
        company_name: rawConsultant.careers[i].company_name,
        profession: rawConsultant.careers[i].profession,
        office: rawConsultant.careers[i].office
      } as ConsultantCareerDescription)
    }
    consultants.push({
      consultant_id: rawConsultant.consultant_id,
      fee_per_hour_in_yen: rawConsultant.fee_per_hour_in_yen,
      rating: rawConsultant.rating,
      num_of_rated: rawConsultant.num_of_rated,
      careers: careerDescriptions
    } as ConsultantDescription)
  }
  return {
    total: rawResult.total,
    consultants
  } as ConsultantsSearchResult
}
