# namegen

This is a package for name generation in rust using multiple generators to pick the right tool for each job.

My use case for this is a website, but I hope this library will be useful to more people that want a quick way to generate names.

## Goals

- High generation performance. Adding samples needs to be correct, but need not perform as well.
- Support multiple generators that suits different sample sets. Markov is great and low maintenance, but only shines if you have a hundred samples. A context free grammar can get out of hand fast, but if you treat it right, you'll get thousands of names with the cost of a dozen samples.
- Offer an API to work on the generators directly as well as a simplified interface to generate full names.
- Allow a user-provided `rand::Rng` on any level of the API.

## Generators

### `markov`
Generate names using a second-order markov chain. This is not a naive implementation, however, and some constraints has been made to create more faithful names:

- Beginnings, middles and ends are not treated as the same type of node, and the name's length is picked at the start of generation.
- A token-frequency restriction can be put on it to prevent tokens occuring more in generated names than any of the samples.