# Recovering Bun/WebKit's `Math.random()` seed

Utilities to brute-force the underlying seed by the weak random generator of
`JavaScriptCore`'s `Math.random()`. This is notably the random number generation
used by Bun and WebKit (Safari, all iOS versions of browsers).

## Background

I got nerd-sniped into investigating the `Math.random()` default seeding
implementation of the [Bun](https://bun.com/) JS runtime (for
*cough cough [reasons](https://github.com/JesseEmond/blitz-2024-registration/)*).

### Where does the seed come from?
My previous investigation like this was [in V8](https://github.com/JesseEmond/blitz-2024-registration/tree/main#mathrandom), where, depite `Math.random()` being a known
weak random generator, the seed still comes from good entropy and is [64-bits](https://github.com/nodejs/node/blob/0a18e136b4a1e860bb2befcbd1f78661ed5fb5e7/deps/v8/src/base/utils/random-number-generator.h#L46).

While looking at Bun, I chased down its initial random seed through this path:
- [`jsc.getRandomSeed`](https://bun.com/reference/bun/jsc/getRandomSeed),
  mentions the seed being set when it starts;
- Connected to its C++ implementation `functionGetRandomSeed` [here](https://github.com/oven-sh/bun/blob/abc26b727b3fe596e11d54113c94e5a1e839b51b/src/jsc/modules/BunJSCModule.h#L1020);
- Implemented as a call to `globalObject->weakRandom().seed()` [here](https://github.com/oven-sh/bun/blob/abc26b727b3fe596e11d54113c94e5a1e839b51b/src/jsc/modules/BunJSCModule.h#L525) (note: appropriate name for `Math.random()`, I appreciate the explicit name to discourage misuse!);
- `globalObject` is a `JSGlobalObject`. Reading on Bun's architecture, we learn
  that it leverages Apple's JavaScriptCore (JSC) for its core runtime engine.
- Bun uses [its fork of webkit](https://github.com/oven-sh/WebKit) for the
  runtime implementation (see [contributing instructions](https://bun.com/docs/project/contributing#building-webkit-locally-debug-mode-of-jsc));
- `weakRandom` is a `WeakRandom` object (see [here](https://github.com/oven-sh/WebKit/blob/74650443cb1a41519624470b386c850c1927762b/Source/JavaScriptCore/runtime/JSGlobalObject.h#L1356));
- [`WeakRandom`'s implementation](https://github.com/oven-sh/WebKit/blob/74650443cb1a41519624470b386c850c1927762b/Source/WTF/wtf/WeakRandom.h#L42) sits in `WTF/wtf/WeakRandom.h` (FYI: `WTF` here [stands for](https://stackoverflow.com/questions/834179/wtf-does-wtf-represent-in-the-webkit-code-base) `Web Template Framework` and not what you thought);
- Its seed by default comes from a cryptographically random number, but can be
  passed as an argument -- so let's chase down `m_weakRandom`'s initialization
  on the global object;
- In the cpp implementation, the `m_weakRandom` [is initialized](https://github.com/oven-sh/WebKit/blob/74650443cb1a41519624470b386c850c1927762b/Source/JavaScriptCore/runtime/JSGlobalObject.cpp#L972)
  either with a forced constant seed based on some options (default false), or...
- The seed is set via a cryptographically-strong **_32-bits_ number**!

## Really, 32-bits...?
Now, `Math.random()` is not meant to be secure in V8 either -- see
[_Hacking the javascript lottery_](https://medium.com/independent-security-evaluators/hacking-the-javascript-lottery-80cc437e3b7f#.pbi9112z5), this is certainly not a _real_ issue.
But, I'm still surprised that it wouldn't go to 64-bits to make bruteforceability
less accessible.

To convince myself that I didn't mess up in my chase, I ran the following test:
```sh
# Generate 1M seeds
for i in {1..1000000}; do
  bun -e 'import { getRandomSeed } from "bun:jsc"; console.log(getRandomSeed());' >> /tmp/seeds.txt
done
# (... be patient ...)

# Check how many total seeds we have (`wc -l`), vs. how many unique ones (`sort -u | wc -l`).
# Extract the duplicates (`sort | uniq -cd`).
cat /tmp/seeds.txt | wc -l; sort -u /tmp/seeds.txt | wc -l; sort /tmp/seeds.txt | uniq -cd
```

And I got:
```
1000000
999886
      2 1018798769
      2 1084645150
[ ... snip ... ]
```

with **114 colliding seeds**.

Now, with [birthday problem](https://en.wikipedia.org/wiki/Birthday_problem) maths,
the expected number of collisions (`E[X]`) for 1M (`k`) samples of 32-bits
(`n=2**32`) numbers would be `~= k(k-1) / 2n ~= 1M(1M-1)/(2**33) ~= 116.4`. And we got 114!

**Looks like we _do_ have a 32-bit seed!**

## Bruteforcing 32-bit seeds

To brute-force a seed, all we need:
- Implement `WeakRandom` the same way to match the `Math.random()` output for a
  given seed.
- Try all seeds from `0` to `2**32-1` until our cloned `Math.random()`
  implementation would generate the same.
- ...
- Profit!

32-bits bruteforcing is very accessible on modern hardware.

## Usage

### Recover seed

```sh
$ time cargo run --release 0.09206707202592623
Seed: 1172590173
cargo run --release 0.09206707202592623  1.55s user 0.01s system 110% cpu 1.407 total
```

And indeed:
```sh
$ bun -e 'import { setRandomSeed  } from "bun:jsc"; setRandomSeed(1172590173); console.log(Math.random());'
0.09206707202592623
```

### Predict next output

```sh
$ time cargo run --release -- --next-pred 0.04654924635051105
0.9982287547550598
cargo run --release -- --next-pred 0.04654924635051105  4.28s user 0.02s system 110% cpu 3.885 total
```

And indeed, I recovered a sequence I generated earlier:
```sh
$ bun -e 'console.log(Math.random()); console.log(Math.random());'
0.04654924635051105
0.9982287547550598
```

### As a library

```rust
for seed in 0..=(u32::MAX as u64) {
    let seed = seed as u32;
    let mut rng = WeakRandom::from_seed(seed);
    // Do generations from 'rng' to match the sequences of rng generations from
    // your target to clone, verify outputs match (e.g. if you're not observing
    // the first random output).
}
```

