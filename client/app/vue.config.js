module.exports = {
  configureWebpack: {
    devtool: 'source-map'
  },
  devServer: {
    proxy: {
      '^/api/user': {
        target: 'http://localhost:3000/api/user/',
        logLevel: 'debug',
        pathRewrite: { '^/api/user': '/api/user' }
      },
      '^/api/advisor': {
        target: 'http://localhost:3001/api/advisor/',
        logLevel: 'debug',
        pathRewrite: { '^/api/advisor': '/api/advisor' }
      }
    }
  }
}
