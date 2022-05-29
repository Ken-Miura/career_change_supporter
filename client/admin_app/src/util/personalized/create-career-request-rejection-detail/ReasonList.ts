export function createReasonList (): string[] {
  const list = [] as string[]
  list.push('画像が不鮮明なため')
  list.push('指定外の証明書類が提出されているため')
  list.push('ユーザー情報と提出された証明書類の情報が一致しないため')
  list.push('提出された画像では職務経歴確認をするための情報が不足しているため')
  list.push('マイナンバーがマスキングされていない（隠されていない）ため')
  return list
}
