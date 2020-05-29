const path = require('path')
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  entry: path.join(__dirname, 'public'),
  resolve: {
    extensions: [".ts", ".tsx", ".js", ".jsx", ".wasm"]
  },
  output: {
    path: path.resolve(__dirname, 'public'),
    filename: 'bundle.js'
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: path.join(__dirname, "public/index.html")
    }),
    new WasmPackPlugin({
      crateDirectory: path.join(__dirname)
    })
  ]
}
