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

There's at least three incompatible versions of OpenAPI in the wild. The rust ecosystem
around these is also a bit of a mess, so here's the details of what we support:

1. `v2` is old, but still disappointingly widly used. We use the `openapi` crate to parse this.
   We've currently forked it because the version on crates.io is very old and has a lot of
   problems.
2. `v3` is newer and seems to be the most commonly used. We use `openapiv3` to parse these
   (The `openapi` crate also has a `v3_0` module, but `openapi` seems less maintained so we're
   using `openapiv3` instead.) We have forked `openapiv3` in order to support `v3.1` below,
   but the version of `openapiv3` on crates.io is sufficient for our `v3` support.
3. `v3.1` is the newest. Despite what the name might suggest, it's a breaking change from v3.
   We currently use a fork of `openapiv3` that provides support for this.
