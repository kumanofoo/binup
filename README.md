# binup
Modifying a Binary File

## Build
```Shell
cargo build --release
```

## Build for musl library (static link)
```Shell
rustup target add x86_64-unknown-linux-musl
cargo build --release --target=x86_64-unknown-linux-musl
```

## Pattern file
PATTERN is replaced with REPLACEMENT.
PATTERN and REPLACEMENT data pairs are separeted by a blank line.
```
PATTERN1
REPLACEMENT1

PATTERN2
REPLACEMENT2

PATTERN3
REPLACEMENT3

    :
    :
```
PATTERN and REPLACEMENT are hexadecimal digits represents one byte.

## Examples
*target_file*
```
Order Request

Pork cutlet curry (300g, Std): 4
Pork cutlet curry (400g, L2): 1
Lightly crisped chicken curry (300g, Std): 2
Squid curry (350g, L1): 1
Sweafood curry (300g, L3): 1
Cheese curry (300g, L1): 2
```

*target_file.patch*
```
2C 20 53 74 64 29 3A 20 34 # , Std): 4
2C 20 53 74 64 29 3A 20 33 # , Std): 3

33 30 30 67 2C 20 4C 33 29 # 300g, L3)
31 35 30 67 2C 20 4C 32 29 # 150g, L2)

67 2C 20 4C 31 29 3A 20 32 # g, L1): 2
67 2C 20 4C 31 29 3A 20 33 # g, L1): 3
```
'#' to the end of the line is a comment.

```Shell
$ cargo run --release -- target_file target_file.patch -o patched_file
    pattern: 2c 20 53 74 64 29 3a 20 34
replacement: 2c 20 53 74 64 29 3a 20 33

    pattern: 33 30 30 67 2c 20 4c 33 29
replacement: 31 35 30 67 2c 20 4c 32 29

    pattern: 67 2c 20 4c 31 29 3a 20 32
replacement: 67 2c 20 4c 31 29 3a 20 33
$
```

*patched_file*
```
Order Request

Pork cutlet curry (300g, Std): 3
Pork cutlet curry (400g, L2): 1
Lightly crisped chicken curry (300g, Std): 2
Squid curry (350g, L1): 1
Sweafood curry (150g, L2): 1
Cheese curry (300g, L1): 3
```
