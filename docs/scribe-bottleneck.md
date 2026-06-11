# Why Scribe-style provers become I/O-heavy

Modern provers can stream witness data so the entire trace need not stay resident. Once arithmetic is highly optimized, performance often moves from multiplication count to memory hierarchy behavior:

1. witness elements arrive in chunks;
2. product constructions emit intermediate layers;
3. each new layer reads the previous layer and writes another buffer;
4. large instances spill beyond cache and may reach storage or a distributed transport.

The included logical model is intentionally transparent. For `N` field elements of `B` bytes, it models a product tree with `log₂ N + 1` read passes and `log₂ N` write passes. The rational accumulator performs one read pass and retains three field elements of working state.

This model is not a hardware benchmark. Run `snark-lab-benches` for arithmetic timings, then instrument your own prover with resident-set, cache-miss, disk, and network counters. The model's purpose is to make the asymptotic traffic difference visible before implementation details obscure it.
