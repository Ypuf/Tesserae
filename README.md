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
- **References:** `{key}` inside a string gets replaced with that key's value, i.e, `"{proc}://{host}"`. always from root, so: `{db/host}`
- **Indentation:** ignored, just for prettifying

## value types (planned)

- strings: `"text"`
- int: `42`
- ports: `:8080`
- durations: `30sec`
- sizes: `4GiB`
- colours: `#FF6611`


