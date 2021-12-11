import { createStore } from 'vuex'
import { SessionState } from '@/store/SessionChecker'

export default createStore({
  state: {
    sessionState: 'none' as SessionState
  },
  mutations: {
    updateSessionState (state, sessionState: SessionState) {
      state.sessionState = sessionState
    }
  },
  actions: {
  },
  modules: {
  }
})
