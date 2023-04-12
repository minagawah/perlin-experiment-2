const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const dist = path.resolve(__dirname, 'dist');
const isProd = process.env.NODE_ENV === 'production';

module.exports = {
  mode: 'production',
  entry: {
    index: './js/index.js',
  },
  output: {
    path: path.resolve(__dirname, 'dist/js'),
    filename: '[name].[fullhash].js',
  },
  resolve: {
    extensions: ['.js'],
  },
  experiments: {
    asyncWebAssembly: true,
  },
  performance: {
    hints: false,
    maxAssetSize: 512000, // 500 bytes
  },
  devtool: isProd ? 'cheap-source-map' : 'inline-source-map',
  devServer: {
    static: {
      directory: dist,
    },
    port: 8080,
    devMiddleware: {
      writeToDisk: true,
    },
  },
  module: {
    rules: [
      {
        test: /\.m?js?$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader',
        },
      },
      {
        test: /\.css$/,
        use: ['style-loader', 'css-loader', 'postcss-loader'],
      },
    ],
  },
  plugins: [
    new CleanWebpackPlugin(),
    new HtmlWebpackPlugin({
      template: './js/index.html',
      filename: 'index.html',
      filename: '../index.html',
      minify: { collapseWhitespace: false },
    }),
    new WasmPackPlugin({
      crateDirectory: __dirname,
      outDir: path.resolve(__dirname, 'dist/pkg'),
      forceMode: isProd ? 'production' : 'development',
    }),
  ],
};
