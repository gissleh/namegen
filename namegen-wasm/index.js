const wasm = await import("Cargo.toml");

let rust = null;

wasm.then((res) => {
    rust = res;
}).catch(err => {
    console.error("Rust failed to load.")
})

export function load() {
    return wasm;
}

export default class NameGenerator {
    constructor(data) {
        if (data != null) {
            this._gen = rust.WasmNameGenerator.load(JSON.stringify(data));
        } else {
            this._gen = rust.WasmNameGenerator.new();
        }
    }

    get data() {
        return this._gen.data();
    }

    generate(formatName, seed) {
        return this._gen.generate_one(formatName, seed);
    }

    generateMany(formatName, amount, seed) {
        return this._gen.generate_many(formatName, amount, seed);
    }

    addPart(options) {
        return this._gen.add_part(options);
    }

    addFormat(formatName, formatStr) {
        return this._gen.add_format(formatName, formatStr);
    }

    learn(partName, sampleSet) {
        try {
            this._gen.learn(partName, sampleSet)
        } catch(err) {
            throw new Error(err);
        }
    }

    toJSON() {
        return this.data;
    }

    free() {
        this._gen.free();
    }
}
