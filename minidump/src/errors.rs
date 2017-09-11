use symbolic_common;

error_chain! {
    links {
        SymbolicError(symbolic_common::Error, symbolic_common::ErrorKind);
    }

    foreign_links {
        IoError(::std::io::Error);
        GoblinError(::goblin::error::Error);
        GimliError(::gimli::Error);
    }

    // errors {
    //     /// TODO(ja): Describe
    //     DebugSymbolsError(desc: String) {
    //         description("Debug Symbols Error")
    //         display("Debug Symbols Error: {}", &desc)
    //     }
    // }
}
