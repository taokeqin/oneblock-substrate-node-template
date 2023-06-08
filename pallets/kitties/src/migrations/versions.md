# v0

```
    pub struct OldKitty(pub [u8; 16]);
```

# v1

```
	pub struct Kitty {
		pub dna: [u8; 16],
		pub name: [u8; 4],
	}
```

# v2

```
	pub struct Kitty {
		pub dna: [u8; 16],
		pub name: [u8; 4],
        pub color: [u8; 4],
	}
```
