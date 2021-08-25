module.exports = {
  configureWebpack: {
    devtool: 'source-map'
  },
  pages: {
    index: {
      entry: 'src/main.ts',
      title: 'Career Change Suppoter', // TODO: リリース前に変更
    }
  },
  devServer: {
    proxy: {
      '^/api/users': {
        target: 'http://localhost:3000',
        logLevel: 'debug',
      },
      '^/api/advisors': {
        target: 'http://localhost:3001',
        logLevel: 'debug',
      }
    }
  }
}
