import { createStore } from 'vuex'
import { DUMMY_RESULT_MESSAGE } from './mutationTypes'

export type State = {
  dummyResultMessage: string | null
};

// 下記URLにVuexにてTypescriptの型推論を有効にするためにkeyが必要と記載されているが
// このkeyを利用するとjestを用いた単体テストの際、vuexをモック化してもエラーが発生し、テストができないため利用しないようにする
// https://next.vuex.vuejs.org/guide/typescript-support.html#typing-usestore-composition-function
// export const key: InjectionKey<Store<State>> = Symbol('symbol for specifying vuex type')

export default createStore<State>({
  state: {
    dummyResultMessage: null
  },
  mutations: {
    [DUMMY_RESULT_MESSAGE] (state: State, message: string) {
      state.dummyResultMessage = message
    }
  },
  actions: {
  },
  modules: {
  }
})
