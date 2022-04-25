import { Ymd } from '@/util/Ymd'

export type CareerDescription = {
    /* eslint-disable camelcase */
    career_id: number,
    company_name: string,
    contract_type: 'regular' | 'contract' | 'other',
    career_start_date: Ymd,
    career_end_date: Ymd | null
    /* eslint-enable camelcase */
}
