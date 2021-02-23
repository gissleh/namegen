export default class NameGenerator {
    /**
     * Create or load a name generator.
     * @param data Pass data to load the generator from it. NO VALIDATION is applied, so this should only come from
     *        a trusted source.
     */
    constructor(data?: NameData)

    /**
     * This gets the data that can be passed in to recreate this exact generator. It will
     * cross the WASM boundary to grab this data, so use this sparingly.
     */
    get data(): NameData

    /**
     * Generate one name.
     *
     * @param formatName The format name to generate
     * @param seed Optionally, a seed for the generator.
     * @returns A string if the format exists, null otherwise.
     */
    generate(formatName: string, seed?: BigInt): string | null

    /**
     * Generate many names. If you need more than a few names, this method will
     * get them for you without crossing the WASM border more than once.
     *
     * Use case: For a website, generating a dozen of them with this and popping
     * them off an array on subsequent generator presses.
     *
     * @param formatName The format name to generate.
     * @param amount Amount of names to generate.
     * @param seed Optionally, a seed for the generator.
     * @returns An array of strings if it works, null otherwise.
     */
    generateMany(formatName: string, amount: number, seed?: BigInt): string[] | null

    /**
     * Add a new name part. If your IDE is worth its salt, it will hide any options you don't need to
     * care about for the specific name.
     *
     * @param options
     */
    addPart(options: AddPartOptions): void

    /**
     * Add a new formatting rule.
     *
     * Examples:
     * * `{first_name} {last_name}`: The referred name parts with a space between.
     * * `{first}'{clan} {=vas|=nar} {ship}`: The third `{...}` is either one of these two words.
     * * `{:full_name|:first_name}, the {title}`: The first `{...}` chooses between these two PREVIOUS formats.
     *
     * @param formatName
     * @param formatStr
     */
    addFormat(formatName: string, formatStr: string): void

    /**
     * Add a sample set to the generator. Different generators want different samples.
     *
     * * `markov`: SampleSet can only include Word samples. WordWeighted is allowed, but the weight is ignored. Labels are ignored.
     * * `cfgrammar`: SampleSet must only include Tokens samples, and the amount of tokens needs to match Labels. Use `*` for an anon. set.
     * * `wordlist`: SampleSet can include Word and WordWeighted samples. Labels are ignored.
     *
     * @param partName
     * @param sampleSet
     */
    learn(partName: string, sampleSet: SampleSet)

    /**
     * Calls the `data` getter.
     */
    toJSON(): NameData
}

type AddPartOptions =
    | ({ kind: "markov", initialTokens: string[], lrs: boolean, lrm: boolean, lre: boolean, rlf: boolean} & AddPartOptionsCommon)
    | ({ kind: "cfgrammar", initialSubtokens: string[], ral: boolean, rlf: boolean} & AddPartOptionsCommon)
    | ({ kind: "wordlist" } & AddPartOptionsCommon)


interface AddPartOptionsCommon {name: string, formatRules: NamePartFormatRule[]}

/**
 * A SampleSet is a versatile structure for feeding name generators learning data. The sample set should
 * be homogenous.
 */
interface SampleSet {
    labels: string[]
    samples: Sample[]
}

type Sample =
    | { word: string }
    | { wordWeighted: [string, number] }
    | { tokens: string[] }

interface NameData {
    parts: NamePartData[]
    formats: NameFormatData[]
}

interface NameFormatData {
    name: string
    parts: NameFormatPartData[]
}

type NameFormatPartData = { part: number } | { format: number } | { text: string } | { random: NameFormatPartData[] }

interface NamePartData {
    name: string
    generator: NamePartGeneratorEnum
    formatRules: NamePartFormatRule[]
}

type NamePartGeneratorEnum =
    | {markov: MarkovData}
    | {cfgrammar: CFGrammarData}
    | {wordlist: WordListData}

/**
 * The rust enum is showing here.
 */
type NamePartFormatRule =
    | "capitalizeFirst"
    | "capitalizeDefault"
    | {capitalizeAfter: string}
    | {removeChar: string}
    | {replaceChar: {from: string, to: string}}

interface MarkovData {
    tokens: string[]
    maxTokens: number[]
    starts: MarkovStartData[]
    totalStarts: number[]
    nodes: MarkovNodeData[]
    lengths: number[]
    totalLengths: number
    lrs: boolean
    lrm: boolean
    lre: boolean
    rtf: boolean
}

interface MarkovStartData {
    /**
     * The tokens to start with.
     */
    t: [number, number]
    /**
     * Weight, based on frequency in sample data.
     */
    w: number
    /**
     * The length if restrictions apply.
     */
    l?: number
    /**
     * The children (indices in nodes array)
     */
    c: number[]
}

interface MarkovNodeData {
    /**
     * Parents (sibling indices)
     */
    p: number[]
    /**
     * Token index
     */
    t: number
    /**
     * Node's weight
     */
    w: number
    /**
     * The length of the sample set (if restrictions apply).
     */
    l?: number
    /**
     * Children (sibling indices, omitted if empty)
     */
    c?: number[]
}

interface CFGrammarData {

}

interface WordListData {

}