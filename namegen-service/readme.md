# namegen-service

This is a simple web service meant to be an isolated agent within a greater system. It can generate names, learn
samples and do other things. There is no security and the data is ephemeral. There is a .wasm build that you can
use to generate the names on the client, this is meant for compiling.

The `NameData`'s version field is incremented with any action and is passed when posting a name. Do with that
what you will.