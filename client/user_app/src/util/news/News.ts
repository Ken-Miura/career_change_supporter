import { Ymd } from '../Ymd'

export type News = {
    /* eslint-disable camelcase */
    news_id: number,
    title: string,
    body: string,
    published_date_in_jst: Ymd,
    /* eslint-enable camelcase */
}
