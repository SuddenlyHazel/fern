pub mod bindings {
    //! This module contains generated code for implementing
    //! the `adder` world in `wit/world.wit`.
    //!
    //! The `path` option is actually not required,
    //! as by default `wit_bindgen::generate` will look
    //! for a top-level `wit` directory and use the files
    //! (and interfaces/worlds) there-in.
    wit_bindgen::generate!({
        generate_all,
        pub_export_macro: true,
        async: true,
    });
    // In the lines below we use the generated `export!()` macro re-use and
}

pub mod sqlite;