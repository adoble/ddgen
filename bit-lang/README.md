# bit-lang

A language parser for specifying bits in a set of words.

## Language specification

### Single bits
To refer to a single bit in a word use:
```
w[b]
```
where `w` is the word index and `b` is the bit index. Both `w` and `b` are 0 based.

For instance, to represent the bit  4in  word 3 use:
```
3[4]
```
Word indexes default to 0 if not specified so:
```
[5] == 0[5]
```
( `==` means equivant to).

If the word is not specified then the square bracket can be omitted:

```
5 == [5] == 0[5]
```

Note that all the above forms can be used.

### Bit Ranges

To refer to a range of bits use:
```
w[a..b]
```
where `a` is the first bit and `b` is the last bit **inclusive**.

For instance, to refer to bit 3 to 6 inclusive in the 2 word use:
```
2[3..6]
```
As for single bits, if word indexes are 0 then they do not need to be specified and neither do the square brackets:
```
3.6 == [3..6] == 0[3..6]
```
Note that all the above forms can be used.

### Whole Words

A whole word can be specifed an emtpy range:
```
w[]
```
Refers to all thre bits in word `w`.

For instance to refers all bits in word 5 use:
```
5[]
```
To refer to the whole of word `0`  use one of the following:
```
` [] == 0[]
```
### Word Ranges

To refer to a range of bits over more then one consecutive word use:
```
w[a]..v[b]
```
This refers to a set of bits from bit `a` in word `w` to bit `b`in word `v`.

Examples:
```
3[4]..6[2]
```
Refers to all the bits from bit 4 in word 3 to bit 2 in word 6.


As before an empty bit range refers to the whole word:
```
3[]..4[]
```
Refers to all the bits in word 3 and 4 (e.g a value over two words).

### Repeating Words

To specify that as word repeats there are a number of opions:

#### Fixed Number of Repeats

The following specifies all bits in a fixed number of words:
```
w[];n
```
Where 'n` is the number of words.

For instance to specify 48 complete words from word 3, use:
```
3[];48
```

#### Variable Number of Repeats

The number of words is often given by a fields that comes before the repeat. This can be specifed by:
```
w[];(v[])⁑n
```

Where `v` i the word containing the number of repeats, ⁑ is a condition and n is number. Conditions allowed are `<` (less then) and `<=` (less than or equal). Note that is highly recommanded that a limit is set so that any clients can set maximum buffer sizes.

For instance, if word 2 contains the number of repeated words and this is followed by the repeated word up to a max of 48 then use:
```
3[];(2[])<49
```


Alternativly one could another condition to mean the same thing:
```
3[];(2[])<=48
```

#### Literals
The actual state of the bits can be set using a literal. This can be shown with the following examples:
- Using hexadecimal to set word 0
```
 [0x23FF]
```
- Using binary to set word 5
```
5[0b1101_0001]
```

## Example Code
```rust
use bit_lang::parser::{BitRange, BitSpec, Condition, Repeat, Word};

fn main() {
    let data = "5[3..7]";
    let bit_spec = bit_lang::parse(data).unwrap();

    assert_eq!( bit_spec.start.index, 5);
    assert_eq!( bit_spec.start.bit_range, BitRange::Range(3,7));
}
```
