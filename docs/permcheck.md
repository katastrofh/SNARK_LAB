# Transcript-bound product and rational PermCheck

The Rust crate checks multisets of tagged values. Each row is compressed as

```text
valueᵢ + β · tagᵢ + γ,
```

where both columns are bound to a domain-separated Merlin transcript before `β` and `γ` are derived.

## Product fingerprint

```text
Πᵢ (valueᵢ + β tagᵢ + γ).
```

## Rational fingerprint

```text
Σᵢ 1/(valueᵢ + β tagᵢ + γ).
```

The rational path returns an explicit `Pole` error if a denominator is zero. In a large field this event is negligible for honest data, but protocol behavior is still defined rather than relying on a panic. Production integrations can resample in an outer protocol with an explicitly domain-separated retry counter, or prove denominator nonzero constraints.

These are transcript-correct fingerprint checks, not yet commitment-backed arguments. A future oracle layer must bind committed columns and prove the product/rational identity inside the chosen polynomial protocol.
