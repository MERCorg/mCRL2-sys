use mcrl2_sys::data::ffi::mcrl2_data_specification_from_string;

fn main() {
    let spec = mcrl2_data_specification_from_string("
        map f: Nat -> Nat;

        var x: Nat;
        eqn f(x) = x + 1;
    ");

    let _rewriter = mcrl2_sys::data::ffi::mcrl2_create_rewriter_jitty(&spec);  
}