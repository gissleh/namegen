# `namegen`

This library offers high performance random name generation with sub-microsecond full names on even older computers. My
use case for this is a website, but I've left this general enough to even fit into other procedural generation systems.

## Features
- `serde` support with feature flag `serde`
- `wasm_bindgen` supported. The github repo has a project for building it.

## Example

## Formats
The name formats has a special syntax. They're what describes how to build a full name from the parts. The tokens are
in curly braces, and they mean the following.

* `{first_name}`: Generate part with name `first_name`.
* `{=stuff}`: Returns the word "stuff".
* `{first_name|=Unnamed}`: A `|` indicates a random pick between the items. Here it will either generate the `first_name`
  part or just the word Unnamed.
* `{:full_name}`: The `:` prefix denotes a format. It can only refer to formats that were added before it, both due to
  optimization and to avoid an infinite recursion.

Here are a few examples.

* `{first_name} {last_name}`: The referred name parts with a space between.
* `{first}'{clan} {=vas|=nar} {ship}`: The third `{...}` is either one of these two.
* `{:full_name|:first_name}, the {title}`: The first `{...}` chooses between these two formats.

## Generators

### `markov`
Generate names using a second-order markov chain. This is not a naive implementation, however, and some constraints has been made to create more faithful names:

- Beginnings, middles and ends are not treated as the same type of node, and the name's length is picked at the start of generation.
- A token-frequency restriction can be put on it to prevent tokens occuring more in generated names than any of the samples.

### `cfgrammar`
Generate names using a context-free grammar, with some constraints to keep symbol frequencies in check and to deal with
those damn `y`s.

- The depth is fixed, so result rules breaks up into token rules that then gets you the token.

### `wordlist`
A simple word list generator, for the cases where output should be one of the samples. The samples can be weighted.
