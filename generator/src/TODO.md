  # TODO

  - Add some more integration tests
  - Common structure serialisation
  - Common structure file names and struct name could confiict with common file names.
    Move them to a seperate module to avoid this.
  - The bit_spec definition for a common structure can lead to conflicts  with how the specification writer
    defines the bit_spec for a command. For instance:
    ```toml
        [commands.TUNE.request]
        a_header = {bits = "0[]", type = "header"}
        [struct.header]
        status = {bits = "0[]"}
        extra_status = {bits = "1[]"}
    ```
    Need to think about this. Consider that the position where a struct is placed in the bit stream
    has to be specified. Maybe need a position  attribute, e.g. `a_header = {struct = "header", position = 0}?`.
    And also checks that enough space is left.
   - Replace members HashMap with (the crate) BiMap. This simplifies the code by removing the need to
     construct the symbol table in Members.
  - Add a license to the manifest (see  https://doc.rust-lang.org/cargo/reference/manifest.html#the-license-and-license-file-fields)
  - Run clippy
  - Incorrect bit_specs seem to be processed without giving an error leading to incorrectly generated code
    ( e.g. incorrect syntax with bit fields)
  - The generated Cargo.toml file needs to have means to update the depedendency version numbers.
  - Often need to pass `HashMap<String, Field>` into functions as this forms the symbol table.
    Could change this to a new type (e.g. `SymbolTable(<HashMap<String, Field>)`), but need to
    see if this can be done easily with serde (see https://github.com/softprops/dynomite/pull/145).
   - Merge field::`TargetType` with `bit_spec::BitSpecType`
   - Move `common/src/test/..` into `common/tests` (as was domn for generator)
   - Ensure that all types are handled in serailize and deserialize ( a lot of impl fors)
   - Read this  https://mmapped.blog/posts/12-rust-error-handling.html and then redesign the error handling
   - Need to handle big endian encodings
   - Replace the comment generation with something like this:  https://github.com/udoprog/genco/issues/53#issuecomment-1821318498
   - Move the type Enumeration out of lib.rs and into a new file. Move over any functions that handle enumeration.
   - Is access.rs required?