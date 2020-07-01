# `namegen`

This library offers high performance random name generation with sub-microsecond full names on even older computers. My
use case for this is a website, but I've left this general enough to even fit into other procedural generation systems.

## Features
- `serde` support with feature flag `serde`

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