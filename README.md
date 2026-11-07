# Tesserae

typechecked config file format i'm making for funsies. overlays or the *tesserae* of this format are the main reason i'm making it

## example

in crates/tesserae/file/example.tess

the idea is: 

load it with the `prod` tessera and `url` becomes `https://ypuf.xyz:25565/api`.

load it with no env and `url` is the root value, in this case `http://localhost:5432/api`.

## syntax

- **Comments:** `//` outside a string, at the start of a line or preceded by a whitespace 
- **Keys:** `key = value`. use `/` for paths, like `db/host`. names can use letters, numbers, `_` and `-`
- **Sections:** `:: name` sets the path for the keys below it. a bare `::` returns to root
- **Tesserae:** `@name` starts a tessera, a block of overrides that only applies when you load its name. a bare `@` returns to root. either way, you also go back to the root section
- **References:** `{key}` inside a string gets replaced with that key's value, e.g, `"{proc}://{host}"`. always from root, so: `{db/host}`
- **Indentation:** ignored, just for prettifying

a string that's only a reference keeps the type of whatever it points at, so `"{port}"` is a port, but `"x{port}"` is a string.

## value types

- strings: `"text"`
- int: `42`
- float: `67.6`
- ports: `:8080`
- durations: `30sec`
- sizes: `4GiB`
- colours: `#FF6611`, or 8 digits for alpha: `#FF6611CC`
- boolean: `true`
- lists: `[1, 2]`


### valid units

- durations: `sec`, `min`, `hr`, `d`, `wk`, `mo` (30 days), `yr` (365 days)
- sizes: `B`, `kB`/`KB`, `MB`, `GB`, `TB`, `KiB`, `MiB`, `GiB`, `TiB`

durations and sizes can be fractional (`1.5hr`) but not negative

`size` gives you bytes, and `duration` gives you a `std::time::Duration`

## philosophy

tesserae is meant to have a sort of similar spirit to toml, except with values that explain themselves, plus environment overlays that are built in.

this will never become CUE or Pkl; the goal is to make a small config format where one file covers every environment, with some rich literals for extra readability

### tesserae are the foundation

almost all of my projects need dev and prod versions of the same config, with some needing staging as well. for me this means essentially duplicate files or a bunch of `.env` files. either way, it annoys me.

the base is the default, this would usually be your "dev" environment. a tessera (@prod) overrides that to whatever you want in that environment.

a tessera can only override keys that already exist at the base level. that way, if you typo a key in `@prod`, you get an error instead of not being able to find out why your stuff doesnt work.

### why i have references

basically, i feel theyre necessary to overlays despite my general lack of interest in evaluation beyond typing

without them, anything built out of other values has to be written out again in every tessera. in the example, `url` is made from `proc`, `host` and `port`, so `@prod` only overrides those and `url` follows along.


---

at the very least, i know i'll use this.

---

must run:

```
cargo fmt && cargo clippy --workspace && cargo test
```
