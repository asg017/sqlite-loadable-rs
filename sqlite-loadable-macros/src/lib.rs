use proc_macro2::Ident;

use syn::{parse_macro_input, spanned::Spanned, Item};

use proc_macro::TokenStream;
use quote::quote_spanned;

/// Wraps an entrypoint function to expose an unsafe extern "C" function of the same name. 
#[proc_macro_attribute]
pub fn sqlite_entrypoint(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as syn::Item);
    match ast {
        Item::Fn(mut func) => {
            let c_entrypoint = func.sig.ident.clone();

            let original_funcname = func.sig.ident.to_string();
            func.sig.ident = Ident::new(
                format!("_{}", original_funcname).as_str(),
                func.sig.ident.span(),
            );

            let prefixed_original_function = func.sig.ident.clone();

            quote_spanned! {func.span()=>
                #func

                /// # Safety
                ///
                /// Should only be called by underlying SQLite C APIs, 
                /// like sqlite3_auto_extension and sqlite3_cancel_auto_extension.
                #[no_mangle]
                pub unsafe extern "C" fn #c_entrypoint(
                    db: *mut sqlite3,
                    pz_err_msg: *mut *mut c_char,
                    p_api: *mut sqlite3_api_routines,
                ) -> c_uint {
                    register_entrypoint(db, pz_err_msg, p_api, #prefixed_original_function)
                }


            }
            .into()
        }
        _ => panic!("Only function items are allowed on sqlite_entrypoint"),
    }
}
