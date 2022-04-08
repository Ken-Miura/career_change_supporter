export function createReasonList (): string[] {
  const list = [] as string[]
  list.push('画像が不鮮明なため')
  list.push('指定外の身分証明書が提出されているため')
  list.push('身分証明書に記載されている内容と提出された内容が一致しないため')
  list.push('提出された画像では本人確認するための情報が不足しているため')
  list.push('マイナンバーがマスキングされていない（隠されていない）ため')
  list.push('運転免許証の裏面の画像がないため')
  return list
}
