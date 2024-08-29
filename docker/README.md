Source: https://github.com/tweag/rust-alpine-mimalloc
Explanation: https://www.tweag.io/blog/2023-08-10-rust-static-link-with-mimalloc/

`build.sh` & `mimalloc.diff` were copy-pasted from e0720a1 (last commit as of 2024-08-29).

The short version is that the musl allocator behaves poorly in multi-threaded scenarios. So the author patched musl to use mimalloc. Now on my machine build times are comparable to those without docker instead of spending an absurd amount of time in kernel threads.

gateway docker build went from 15min to 5min and cli went from 19 to 6 in the CI with this fix.
