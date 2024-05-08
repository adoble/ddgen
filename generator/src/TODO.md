  # TODO
  [ ] Variable fields without a dependency, e.g `5[];<100`
  [ ] This `data[1..=1].serialize_word(self.a_u8);` does work compile 
  [ ] Ensure that all types are handled in serailize and deserialize ( a lot of impl fors)
  [ ] Add some more integration tests
  [ ] Common structures cannot be variable length. Also varabiable repeating arrays need to be at the end of 
    the data stream. All of these problems woudl go away if we had labeled fields and follows field syntax
    in bit_lang.
  [ ] Common structure file names and struct name could confiict with common file names.
    Move them to a seperate module to avoid this.
  [ ] The bit_spec definition for a common structure can lead to conflicts  with how the specification writer
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
  [ ] Bit spec with fixed repeating range of 0 should not be allowed   
  [ ] Run clippy
  [ ] Incorrect bit_specs seem to be processed without giving an error leading to incorrectly generated code
    ( e.g. incorrect syntax with bit fields)
  [ ] The generated Cargo.toml file needs to have means to update the depedendency version numbers.
  [ ] Merge field::`TargetType` with `bit_spec::BitSpecType`
   [ ] Move `common/src/test/..` into `common/tests` (as was done for generator)
   [ ] Read this  https://mmapped.blog/posts/12-rust-error-handling.html and then redesign the error handling
   [ ] Need to handle big endian encodings
   [ ] Replace the comment generation with something like this:  https://github.com/udoprog/genco/issues/53#issuecomment-1821318498
   [ ] Move the type Enumeration out of lib.rs and into a new file. Move over any functions that handle enumeration.
   [ ] Is access.rs required?
   [ ] A field is not always an enum, but a value with a limited range (e.g. 0-15). Need a way to specify the range.