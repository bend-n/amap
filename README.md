# amap

Simple array initialization macro.

## Ever wanted to create a const `HashMap<usize, T>`, and started thinking, wouldn't it be nice if this was a array?

No?

Well now you can!
Its as simple as

```rust
amap! {
  4 => 56,
  2 => 32,
} // creates a [Option<i32>; 5] for all your indexing needs
```

### Think it would be too much boilerplate to have multiple keys for one value?

Patterns got you covered!

```rust
amap! {
  0..=4 => 2,
  5 | 6 => 3,
}
```

### Want to put it in a constant? No problem!

It's just a array!

```rust
const ID_MAP: [Option<i32>; 6] = amap! {
  5 => 6,
  2 => 1,
}
```
