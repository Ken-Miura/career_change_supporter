import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetConsultantDetailResp } from './GetConsultantDetailResp'
import { ConsultantDetail } from './ConsultantDetail'
import { ConsultantDetail as RawConsultantDetail } from './raw-response/ConsultantDetail'
import { ConsultantCareerDetail } from './ConsultantCareerDetail'

export async function getConsultantDetail (consultantId: string): Promise<GetConsultantDetailResp | ApiErrorResp> {
  const params = { consultant_id: consultantId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/api/consultant-detail?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const rawResult = await response.json() as RawConsultantDetail
  const result = convertResult(rawResult)
  return GetConsultantDetailResp.create(result)
}

// ConsultantDetailのcareersの要素内に一意に識別できる値は含まれていない。
// v-forで回す際に一意な識別子が必要になるので、返ってきたレスポンスを画面に表示する前に一意な識別子を生成して含めておく
function convertResult (rawResult: RawConsultantDetail): ConsultantDetail {
  const careers = [] as ConsultantCareerDetail[]
  for (let i = 0; i < rawResult.careers.length; i++) {
    const consultantCareerDetail = {
      counsultant_career_detail_id: i,
      company_name: rawResult.careers[i].company_name,
      department_name: rawResult.careers[i].department_name,
      office: rawResult.careers[i].office,
      years_of_service: rawResult.careers[i].years_of_service,
      employed: rawResult.careers[i].employed,
      contract_type: rawResult.careers[i].contract_type,
      profession: rawResult.careers[i].profession,
      annual_income_in_man_yen: rawResult.careers[i].annual_income_in_man_yen,
      is_manager: rawResult.careers[i].is_manager,
      position_name: rawResult.careers[i].position_name,
      is_new_graduate: rawResult.careers[i].is_new_graduate,
      note: rawResult.careers[i].note
    } as ConsultantCareerDetail
    careers.push(consultantCareerDetail)
  }
  return {
    consultant_id: rawResult.consultant_id,
    fee_per_hour_in_yen: rawResult.fee_per_hour_in_yen,
    rating: rawResult.rating,
    num_of_rated: rawResult.num_of_rated,
    careers
  } as ConsultantDetail
}
