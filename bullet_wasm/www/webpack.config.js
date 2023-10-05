const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  mode: "development",
  devtool: 'inline-source-map',
  plugins: [
    new CopyWebpackPlugin(['index.html'])
  ],
  devServer: {
    proxy: {
      '/api': 'http://localhost:4000'
    }
  }
};
