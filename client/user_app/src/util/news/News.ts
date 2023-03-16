import { Ymd } from '../Ymd'

export type News = {
    /* eslint-disable camelcase */
    title: string,
    body: string,
    published_date_in_jst: Ymd,
    /* eslint-enable camelcase */
}
