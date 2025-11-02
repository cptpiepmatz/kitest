use std::mem;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn);
    let name = &func.sig.ident;
    let func_name_str = name.to_string();
    let name_uppercase = format_ident!("{}", func_name_str.to_uppercase());

    let mut experimental = false;
    let mut flaky = false;
    for attr in mem::take(&mut func.attrs) {
        if attr.path().is_ident("experimental") {
            experimental = true;
        }
        if attr.path().is_ident("flaky") {
            flaky = true;
        }
    }

    let expanded = quote! {
        mod #name {
            #[linkme::distributed_slice(crate::TESTS)]
            static #name_uppercase: kitest::test::Test<crate::Extra> = kitest::test::Test::new(
                kitest::test::TestFnHandle::Ptr(#name),
                kitest::test::TestMeta {
                    name: std::borrow::Cow::Borrowed(#func_name_str),
                    ignore: kitest::ignore::IgnoreStatus::Run,
                    should_panic: kitest::panic::PanicExpectation::ShouldNotPanic,
                    extra: crate::Extra {
                        experimental: #experimental,
                        flaky: #flaky,
                    }
                }
            );

            fn #name() -> kitest::test::TestResult {
                super::#name().into()
            }
        }

        #func
    };

    expanded.into()
}
