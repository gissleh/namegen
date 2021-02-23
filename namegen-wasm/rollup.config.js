import rust from "@wasm-tool/rollup-plugin-rust";

export default {
    input: {
        namegen_wasm: "Cargo.toml",
    },
    plugins: [
        rust(),
    ],
};
