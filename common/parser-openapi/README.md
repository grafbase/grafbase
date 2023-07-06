Parses an OpenAPI spec and generates a schema for that API.

### Layout of This Crate

This crate is divided into a few modules:

- `parsing` is responsible for calling out to parser crates to get an in memory representation
  of an OpenAPI document. It takes this structure and uses it to populate the graph structure
  provided by the `graph` module.
- The `graph` module is the core of this crate.
  - It provides a specification version agnostic IR for us to work with. OpenAPI schemas form
    a graph, so it's very handy to use a graph library to represent them. It lets us traverse
    the graph in both directions, run graph algorithms on the schema etc. etc.
  - This module also provides a lot of types that abstract over the graph representation, so
    consumers of the module have nice domain specific types to work with.
  - It also provides some helper functionality for building & transforming the graph, which is
    used as part of the `parsing` process.
- The `validation` module runs after `parsing` has built up the graph, and applies additional
  validation to the input.
- Finally, the `output` module works with the types provided by `graph` to build a `Registry`
  that's used by the rest of our system

### Versions of OpenAPI

There's at least three incompatible versions of OpenAPI in the wild.

1. `v2` is old, but still disappointingly widly used. We use the `openapi` crate to parse this.
2. `v3` is newer and seems to be the most commonly used. We use `openapiv3` to parse these
   (The `openapi` crate also has a `v3_0` module, but `openapi` seems less maintained so we're
   using `openapiv3` instead.)
3. `v3.1` is the newest. Despite what the name might suggest, it's a breaking change from v3.
   At the time of writing we don't support this.
