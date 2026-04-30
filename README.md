# Syntax (syntax-core)

🌲 SYNTAX v 0.0.4 

"Fast", multi-strategy parsing and transformation kit for Rust. Foundational framework for creating predictable parsers.

Originally designed as a pure AST (until I realized AST isn't the optimal model for all languages), Syntax provides a service layer over available structures:

- `Ast` for structure-first languages
- `Plan` for execution-oriented languages 
- `Tape` for lazy document traversal 
- `Others` other standard formats on backburner

*officially "syntax-core" because syntax is a reserved name.*

Language developers can use the Syntax API to define their shapes and parser rules, and deploy their parser engines in other projects.

"Syntax Houses" are the official family of language-specific adapters provided as standalone lib* crates. Syntax enumerates its official and known crates. Lib-prefixed crates not enumerated in Syntax are not part of the offering.

# Syntax Dev Roadmap - internal

# Houses Dev Roadmap  
Planned syntax based libs, we'll see how far we get. 

## Tier 1 - Now
- `libjson` (in progress)
- `libhtml` (in progress) 

## Tier 2 - Soon
- `libcss` (in progress)
- `libtpl` (template interpolation)
- `libexpr` (expressions)


## Tier 3 - Later
- `libtoml`
- `libyuml` (yaml compat -- libyaml was taken)

## Tier 4 - Nice to haves
- `libpython`
- `libjs`
- `libbash`
- `libsqlx` (sql -- libsql was taken)

## Make your own Syntax House?

Let me know and I'll add it to the index.


# Testing & Performance

Each crate follows a shared testing and benchmarking practice:

- **Version-tracked benchmarks** - `perf.sh` runs benchmarks and writes results to `perf.tsv`, a registry of parse performance mapped to crate versions. This lets us track regressions and improvements across releases.
- **Differential testing** - where a leading crate exists for a given format, we compare correctness and performance against it. Where no leading crate exists, we validate against the relevant RFC or spec for compatibility and correctness.
- **Parse strategy analysis** - benchmarks cover the different parse strategies (AST, Tape, etc.) to understand trade-offs per format.
- **Tiered test data** - three tiers of test inputs (small/typical, medium/complex, large/adversarial) used for both correctness and performance validation.


# My Architectural Principles (My IBI - Itty Bitty Ities)

- **Composability** : separation of concerns at the module or library level should expose an API/Service Layer and parts should generally be swappable (implied Modularity)
- **Interoperability** : well I guess I just expressed that above, an extension to this is that we create clean partitions along the service lines, for example we don't kitchen sink core modules, instead we add libraries or plugins in a separate offering. Use decoupling patterns like Strategy, Adapter, Plugin, Injection, etc.
- **Accessibility** : things have to make sense to me; esoteric nomenclature or patterns need to be normalized/deconstructed for grokking. I'm a visual thinker so verbose ideas need to collapse into visual symbols I can hang my hat on. Example: SuperDuperParserthing -> Parser -> px. I do this a lot. 
- **Agility** : implied by the upstream ities, all of them together serve this aim. I like optionality, but YAGNI applies.
- **Scalability** : implied by the upstream ities. Get it for free if you meet the others consistently, force it in a refactor when the time is right AND you're willing to pay for it. I am crazy enough to pull the refactor lever if I am not happy with my ities.



# License

License is AGPL-3-only in this and all related projects until I figure out licensing generally. Note: this may change later to dual or triple license. 

(c) Qodeninja and "Qodeninja's Software House" 2026
