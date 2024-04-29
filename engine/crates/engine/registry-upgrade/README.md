Upgrades registries from v1 -> v2. The engine should mostly use v2 at the moment,
but the parsers use v1. Writing a builder for v2 and updating v1 is some work
that I don't want to do for now, so this crate provides an interim solution
