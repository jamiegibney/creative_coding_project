# Navigating the project's source code

### Documentation
Not every part of this project is documented, but many useful objects, methods, and functions are. Some files contain step-by-step comments to explain a process, and others provide documentation comments to explain how a certain function or method should be used. Most documentation is intended to make certain modules easier to use for the programmer using them.

Please note that in Rust, `//` comments are only visible in the source code, but `///` comments are "doc comments" which are exposed by many code editors and in the [project's documentation](LINK).

##### Viewing the project's documentation in the browser
You may view the project's auto-generated documentation in your browser by navigating to the `docs/` folder and opening `index.html`. This may provide an easier way to navigate the public parts of the project, but will not document all source code.

### Main modules
This project contains several main modules:

- `app`: application-related logic and data, such as the audio processing callback, the application's state, the draw loop, etc.
- `dsp`: digital signal processors, ranging from filters and spectral processors to compression and delay.
- `generative`: creative, "generative" algorithms used to control certain parts of the device.
- `gui`: graphical user-interface logic and components — used for the UI controls and spectrograms, for instance.
- `shaders`: GPU shaders for computing certain algorithms fast.
- `util`: utility logic, such as decibel-to-level conversion, interpolation, value smoothing, etc.

Also note these top-level files:

- `prelude.rs`: a file to allow easy access to certain code from anywhere in the program.
- `settings.rs`: program-wide settings, such as the window size or default tempo.

### Third-party libraries
This project relies on many third-party libraries. All of these "crates" are visible in the [`Cargo.toml`](./Cargo.toml) file under the `[dependencies]` section.

### Rust 101

If you're unfamiliar with Rust, here are some key things to bear in mind:

- Variables are declared with `let`, and are immutable by default.
- Mutable variables are declared with `let mut`.
- The primitive types look like `i32`, `f64`, `bool`, `usize`, etc.
- `let` can (and often will) infer the type of a variable, and everything must have a type:
    ```rust
    let a = 78324;  // type: i32
    let b = 0.435;  // type: f64
    let c = false;  // type: bool
    ```
- Variables can be redefined, which is called "overshadowing":
    ```rust
    let value = 42;
    some_function(value);

    let value = 9000;
    some_function(value);
    ```
- **All** blocks surrounded by `{}` are expressions, meaning they can have a value. The value of a `{}` block is often at the end, and is not terminated with a semicolon. For example:
    ```rust
    let number = {
        // unused value
        let unused_number = 42;

        // arbitrary function call
        some_function();

        // value used only within this block
        let important_number = 3.14159;

        // this is the result of the block, and the value of "number"
        important_number * 2.0
    };
    ```
- > This also applies to functions, so `return` is not always required to return values.
- `(a, b)` is a *"tuple"*, which can hold multiple values.
- `[T; N]` is an array which holds `N` values of `T`. Arrays are stack-allocated by default.
- `&[T]` is a *"slice"* of `T` values, which usually means a reference to heap-allocated data. All slices have a `.len()` method.
- `&str` is a *"string slice"*, which is a reference to heap-allocated UTF-8 data.
- The `Box` type is a pointer to owned data on the heap:
    ```rust
    let data = [1.234; 56];     // type: [f64; 56]      stack-allocated
    let boxed = Box::new(data); // type: Box<[f64; 56]> heap-allocated
    ```
- `struct`s can (optionally) hold data and implement methods with `impl`.
- `enum`s have variants, and variants can store data. `enum`s can also implement methods with `impl`.
- `trait`s are a kind of interface — they can define methods which are implemented by `struct`s or `enum`s, and they can restrict what types can be used in certain contexts.
- `T` is owned data, `&T` is immutably-borrowed data, and `&mut T` is mutably borrowed data.
- You can have *any* number of immutable references to data at a time, as long as there are **no mutable references**.
- You can only have one mutable reference to data at a time, as long as there are **no immutable references**.
- `let a = b` will ***move*** the data at `b` into `a`, unless b implements the `Copy` trait. All of the primitive types implement `Copy`.
    - Similarly, a function which takes `T` as an argument will *consume* the data.
    - Because of this move-by-default behaviour, borrowing is used very often.
- `|| {}` is the syntax of a *"closure"*, which is like a lambda function in other languages. Parameters go inside `||`:
    ```rust
    let closure = |a: f32| { a.sqrt() * PI + a.log10() }; // type: impl Fn(f32) -> f32
    ```
- There is no `null` — the `Option` enum is used for that, which has the `Some(T)` and `None` variants.
- `panic` is a term which refers to the program exiting, typically because of an unexpected or unrecoverable action.
- Functions ending with `!` are macros, such as `println!()`, `vec![]`, or `todo!()`.
- A `crate` is a library of Rust code — usually third-party.

There are also some more advanced methods used in this project:

- The `dyn` keyword refers to [dynamic dispatch](https://en.wikipedia.org/wiki/Dynamic_dispatch), and is used in Rust as "trait objects":
    ```rust
    // any trait
    trait SomeTrait {
        fn do_something();
    }

    // an example struct which implements SomeTrait
    struct SomeStruct;
    impl SomeTrait for SomeStruct {
        // implementation...
    }

    // another example struct which implements SomeTrait
    struct SomeOtherStruct;
    impl SomeTrait for SomeOtherStruct {
        // implementation...
    }

    // a struct which needs to hold any type which implements SomeTrait
    struct OtherStruct {
        some_trait_object: Box<dyn SomeTrait>, // this is a "trait object"
    }

    impl OtherStruct {
        pub fn do_something_here(&self) {
            // regardless of which type `some_trait_object` is, the `do_something()` 
            // method of `SomeTrait` can be called, and the correct implementation is
            // located at runtime. this is what dynamic dispatch is useful for!
            self.some_trait_object.do_something();
        }
    }
    ```
- "Atomic" types are types which use atomic operations — operations which complete as whole steps. These are used in multi-threaded contexts to prevent different threads from accessing or interrupting half-complete operations.
- The `Arc` type is a thread-safe, reference counting pointer. "Arc" stands for "Atomically Reference-Counted". It is used to share data amongst threads without needing to clone the data. However, it does not allow the underlying data to be mutated by default.
- `Mutex` and `RwLock` are used to provide thread-safe "interior mutability" via a lock, which allows a value to mutated without a mutable reference to it.
- `RefCell` allows for interior mutability by providing runtime borrow-checking (which is usually performed at compile-time). It is not thread-safe, however.
- Multi-threading is used throughout the project as a way of asynchronously processing certain parts of the program — that is, certain operations are performed alongside the "main thread". A common example where multi-threaded is used is the audio processing, which is done on a separate thread. Accessing data on the audio thread requires the use of thread-safe pointers, or message channels such as `mpsc` in the standard library, or structs from the `triple_buffer` and `crossbeam_channel` crates.
