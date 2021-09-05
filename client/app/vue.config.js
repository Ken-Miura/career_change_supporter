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
      '^/api': {
        target: 'http://localhost:3000',
        logLevel: 'debug',
      },
    }
  }
}
