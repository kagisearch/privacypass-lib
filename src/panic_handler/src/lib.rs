extern crate proc_macro;
use proc_macro::TokenStream;

// -----------------------------------------------------------------------------
// -----------------------  CODE FOR HANDLING PANICS  --------------------------
// -----------------------------------------------------------------------------

// The macros simply make the process of protecting every function less verbose.
// This is the fundamental idea of how this works.
/*
fn panic_handled_function() -> String {
    panic::set_hook(Box::new(|panic_info| {
        print_panic_info(panic_info);
    }));

    let result = panic::catch_unwind(|| -> _ {
        // possibly-panicking code goes here
        let greeting_file_result = File::open("hello.txt");
        panic!("Normal panic");
        Ok()
    });

    // after possible panic was caught, process it
    let res = match result {
        Err(err) => format!("Panic: {:?}", err),
        Ok(res) => match res {
            Ok(ress) => ress,
            Err(errr) => format!("Error: {:?}", errr),
        },
    };
    return format!("res: {:?}", res);
}
*/

// NOTE: no way to handle error generated here. Yet, as all we do is parsing fixed working code,
//       using expect and not handling the panic should be fine

#[proc_macro]
pub fn begin_panic_handling(_: TokenStream) -> TokenStream {
    r#"
    use std::panic;
    // note that we can't get the error message out, but we can at least print it
    panic::set_hook(Box::new(|panic_info| {
        // while can't easily get this message into the resulting error, it goes to stdout
        let mut message = "R: PANIC occurred".to_string();
        if let Some(location) = panic_info.location() {
            message += &format!(" in file '{}' at line {}",
                location.file(),
                location.line(),
            );
        } else {
            message += " in unknown location";
        }
        if let Some(reason) = panic_info.payload().downcast_ref::<&str>() {
            message += &format!(", error message: {}", reason);
        } else {
            message += &format!(", no error message");
        }
        eprintln!("{}", message);
    }));
    "#
    .to_string()
    .parse()
    .expect("Generated invalid tokens within `begin_panic_handling` macro.")
}

#[proc_macro]
pub fn end_panic_handling(_: TokenStream) -> TokenStream {
    r#"
    // after possible panic was caught, process it
    let result = match result {
        Ok(non_panicky_result) => match non_panicky_result {
            Ok(res_cstr) => res_cstr,
            Err(err) => error_json_retval(&format!("{:?}", err))
        },
        Err(_) => error_json_retval("panic")
    };
    "#
    .to_string()
    .parse()
    .expect("Generated invalid tokens within `end_panic_handling` macro.")
}
