import { InjectionKey } from '@vue/runtime-dom'
import { createStore, Store } from 'vuex'

export type State = {
  applyNewPasswordResultMessage: string | null
};

export const key: InjectionKey<Store<State>> = Symbol('symbol for specifying vuex type')

export default createStore<State>({
  state: {
    applyNewPasswordResultMessage: null
  },
  mutations: {
    setApplyNewPasswordResultMessage (state: State, message: string) {
      state.applyNewPasswordResultMessage = message
    }
  },
  actions: {
  },
  modules: {
  }
})
