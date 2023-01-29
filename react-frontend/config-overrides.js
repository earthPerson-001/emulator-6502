const path = require('path');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = function override(config, env) {

    /**
     * Add WASM support
     */

    // Make file-loader ignore WASM files
    const wasmExtensionRegExp = /\.wasm$/;
    config.resolve.extensions.push('.wasm');
    config.module.rules.forEach(rule => {
        (rule.oneOf || []).forEach(oneOf => {
            /**
             * This is very hacky method
             * and should be changed
             */
            if (oneOf.exclude && oneOf.type === 'asset/resource') {
                oneOf.exclude.push(wasmExtensionRegExp);
            }
        });
    });

    // Add a dedicated loader for WASM
    config.module.rules.push({
        test: wasmExtensionRegExp,
        include: path.resolve(__dirname, 'src'),
        use: [{ loader: require.resolve('wasm-loader'), options: {} }]
    });

    // flagging module as WebAssembly module for webpack
    config.experiments = {
        asyncWebAssembly: true,
        syncWebAssembly: true
    }

    // wasm-pack-plugin
    config.plugins.push(new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, "."),   // implies that the crate is in the current directory
        outDir: path.resolve(__dirname, "../wasm-6502") // output directory to wasm binaries and js files (npm module)
    }))

    return config;
};