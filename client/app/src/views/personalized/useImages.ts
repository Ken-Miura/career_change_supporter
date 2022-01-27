import { reactive } from 'vue'

// eslint-disable-next-line
export function useImages () {
  const images = reactive({
    image1: null as File | null,
    image2: null as File | null
  })
  const onImage1StateChange = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      // HTMLInputElement以外が来るときはinputタグ以外に関数が指定されている。
      // inputタグ以外にしていすることは想定していないため、Errorとする。
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    const files = target.files
    // filesがnullのケースは思い当たらないため、その場合は想定外のエラーとして処理する。
    if (files === null) {
      throw new Error(`files === null: ${target}`)
    }
    // ファイルを選択した状態から、もう一度ファイル選択を押す。
    // その後、キャンセルを押すとファイルが選択されていない状態となる。
    // ファイルが選択されていない状態は、files.length === 0となる。
    // こののケースは正常であるため、保持しているFileをnullで更新し、リターンする。
    if (files.length === 0) {
      images.image1 = null
      return
    }
    images.image1 = files[0]
  }
  const onImage2StateChange = (e: Event) => {
    const target = (e && e.target)
    if (!(target instanceof HTMLInputElement)) {
      // HTMLInputElement以外が来るときはinputタグ以外に関数が指定されている。
      // inputタグ以外にしていすることは想定していないため、Errorとする。
      throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
    }
    const files = target.files
    // filesがnullのケースは思い当たらないため、その場合は想定外のエラーとして処理する。
    if (files === null) {
      throw new Error(`files === null: ${target}`)
    }
    // ファイルを選択した状態から、もう一度ファイル選択を押す。
    // その後、キャンセルを押すとファイルが選択されていない状態となる。
    // ファイルが選択されていない状態は、files.length === 0となる。
    // こののケースは正常であるため、保持しているFileをnullで更新し、リターンする。
    if (files.length === 0) {
      images.image2 = null
      return
    }
    images.image2 = files[0]
  }
  return {
    images,
    onImage1StateChange,
    onImage2StateChange
  }
}
