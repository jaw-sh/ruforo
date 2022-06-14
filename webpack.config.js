const path = require('path');
const MiniCssExtractPlugin = require("mini-css-extract-plugin");

module.exports = {
    mode: "development",
    devtool: "eval-source-map",
    plugins: [
        new MiniCssExtractPlugin({
            filename: "[name].css",
        }),
    ],
    module: {
        rules: [
            {
                test: /\.m?js$/,
                exclude: /(node_modules)/,
                use: {
                    // We can't reply on swcpack yet but we can still use swc
                    // `.swcrc` can be used to configure swc
                    loader: "swc-loader"
                }
            },
            {
                test: /\.s?[ac]ss$/i,
                use: [
                    // 
                    MiniCssExtractPlugin.loader,
                    // Translates CSS into CommonJS
                    "css-loader",
                    // Compiles Sass to CSS
                    "sass-loader",
                ],
            }
        ]
    },
    resolve: {
        extensions: ['*', '.js', '.jsx', '.scss']
    },
    entry: {
        chat: path.resolve(__dirname, './resources/js/chat.js'),
        main: path.resolve(__dirname, './resources/js/attachments.js'),
        style: path.resolve(__dirname, './resources/css/main.scss'),
    },
    output: {
        path: path.resolve(__dirname, './public/assets'),
        filename: '[name].js',
    },
    devServer: {
        contentBase: path.resolve(__dirname, './public'),
        hot: true
    },
    devtool: false
};