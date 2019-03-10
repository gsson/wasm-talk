const path  = require('path')

const HtmlWebpackPlugin = require('html-webpack-plugin')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')


const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  entry: "./web/js/main.js",
  output: {
    path: dist,
    filename: "bundle.js"
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: './web/index.html'
    }),

    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "crate"),
      extraArgs: "--no-typescript --release --target browser",
      forceMode: "release"
    }),
  ]
};
