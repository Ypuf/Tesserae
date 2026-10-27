# Tesserae

typechecked config file format i'm making for funsies. overlays or the *tesserae* of this format are the main reason i'm making it

## example

in file/example.tess

the idea is: 

load it with `env="prod"` and `url` becomes `https://ypuf.xyz:25565/api`.

load it with no env and `url` is the root value, in this case `http://localhost:5432/api`.

## syntax (planned)

- **Comments:** `//` outside a string, at the start of a line or preceded by a whitespace 
- **Keys:** `key = value`. use `/` for paths, like `db/host`
- **Sections:** `:: name` sets the path for the keys below it. a bare `::` returns to root
- **Tesserae:** `@name` starts a tessera, a block of overrides that only applies when you load its name. a bare `@` returns to root
- **References:** `{key}` inside a string gets replaced with that key's value, e.g, `"{proc}://{host}"`. always from root, so: `{db/host}`
- **Indentation:** ignored, just for prettifying

## value types (planned)

- strings: `"text"`
- int: `42`
- float: `67.6`
- ports: `:8080`
- durations: `30sec`
- sizes: `4GiB`
- colours: `#FF6611`
- boolean: `true`

## philosophy

tesserae is meant to have a sort of similar spirit to toml, except with values that explain themselves, plus environment overlays that are built in.

this will never become CUE or Pkl; the goal is to make a small config format where one file covers every environment, with some rich literals for extra readability

### tesserae are the foundation

almost all of my projects need dev and prod versions of the same config, with some needing staging as well. for me this means essentially duplicate files or a bunch of `.env` files. either way, it annoys me.

the base is the default, this would usually be your "dev" environment. a tessera (@prod) overrides that to whatever you want in that environment.

a tessera can only override keys that already exist at the base level. that way, if you typo a key in `@prod`, you get an error instead of not being able to find out why shit doesnt work.


### why i have references

basically, i feel theyre necessary to overlays despite my general lack of interest in evaluation beyond typing

its because of the essential idea of i want to make my config be 1 file rather than 2 or 3 for my environments.

// i should finish explaining this, and better




at the very least, i know i'll use this.
