//! `#[rpc]` — one attribute that makes a function reachable from both transports.
//!
//! Tauri offers no way to invoke a command by name from Rust, so exposing the
//! command surface over HTTP means generating a second entry point per command.
//! Doing that by hand would be 345 shims and a standing invitation to drift: the
//! remote surface would quietly become whatever subset somebody remembered to
//! wire up. This macro makes the two entry points a single edit instead.
//!
//! Replace `#[tauri::command]` with `#[rpc]`. The command keeps working over IPC
//! exactly as before, and additionally registers a dispatch entry the daemon can
//! call by name.
//!
//! ```ignore
//! #[rpc]
//! pub async fn coder_list_threads(
//!     service: State<'_, Arc<CoderService>>,
//! ) -> Result<Vec<CoderThread>, String> { .. }
//! ```
//!
//! Parameters are classified by type:
//!
//! - `State<'_, T>` — resolved from Tauri managed state, same instance the IPC
//!   path would get
//! - `AppHandle` — the running handle
//! - anything else — deserialised from the JSON argument object, by parameter
//!   name, accepting the camelCase spelling the frontend sends as well as the
//!   snake_case one
//!
//! A command taking `Window`/`WebviewWindow` cannot be served without a window,
//! so it must say so with `#[rpc(local_only)]` rather than be silently dropped
//! from the remote surface.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, PathArguments, ReturnType, Type};

#[proc_macro_attribute]
pub fn rpc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let local_only = !attr.is_empty() && attr.to_string().replace(' ', "") == "local_only";
    if !attr.is_empty() && !local_only {
        return compile_error(
            proc_macro2::Span::call_site(),
            "the only supported argument is `local_only`",
        );
    }

    let func = parse_macro_input!(item as ItemFn);

    if local_only {
        // IPC only, on purpose. Still tagged so a reader can see the surface was
        // considered rather than forgotten.
        return quote!(#[tauri::command] #func).into();
    }

    match expand(&func) {
        Ok(glue) => quote!(#[tauri::command] #func #glue).into(),
        Err(err) => err.into(),
    }
}

/// How one parameter is produced when the call arrives over HTTP.
enum Source {
    /// `State<'_, T>` — pulled from managed state.
    State(Box<Type>),
    /// The `AppHandle` itself.
    Handle,
    /// A JSON field, by parameter name.
    Arg { snake: String, camel: String },
}

fn expand(func: &ItemFn) -> Result<proc_macro2::TokenStream, TokenStream> {
    let name = &func.sig.ident;
    let name_str = name.to_string();
    let is_async = func.sig.asyncness.is_some();

    let mut bindings = Vec::new();
    let mut call_args = Vec::new();

    for (index, input) in func.sig.inputs.iter().enumerate() {
        let FnArg::Typed(typed) = input else {
            return Err(compile_error(
                span_of(input),
                "#[rpc] does not support methods taking self",
            ));
        };

        let binding = format_ident!("arg{}", index);
        let source = classify(&typed.ty, &typed.pat).map_err(|msg| compile_error(span_of(input), &msg))?;

        bindings.push(match source {
            Source::State(inner) => quote! {
                let #binding: ::tauri::State<'_, #inner> = ::tauri::Manager::state::<#inner>(app);
            },
            Source::Handle => quote! {
                let #binding = ::core::clone::Clone::clone(app);
            },
            Source::Arg { snake, camel } => quote! {
                // Absent reads as `null`, which deserialises for `Option<T>` and
                // is rejected with a useful message for anything else.
                let #binding = ::serde_json::from_value(
                    args.get(#snake).or_else(|| args.get(#camel)).cloned()
                        .unwrap_or(::serde_json::Value::Null),
                )
                .map_err(|e| format!("{}: bad argument `{}`: {}", #name_str, #snake, e))?;
            },
        });
        call_args.push(quote!(#binding));
    }

    let invoke = if is_async {
        quote!(#name(#(#call_args),*).await)
    } else {
        quote!(#name(#(#call_args),*))
    };

    // `Result<T, E>` is the overwhelming shape; a plain return is accepted too so
    // infallible commands do not have to be dressed up as fallible ones.
    let produce = if returns_result(&func.sig.output) {
        quote! {
            let value = #invoke.map_err(|e| ::std::string::ToString::to_string(&e))?;
        }
    } else {
        quote! { let value = #invoke; }
    };

    let register = format_ident!("__rpc_register_{}", name);

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #register {
            use super::*;

            fn call(
                app: &::tauri::AppHandle,
                args: ::serde_json::Value,
            ) -> ::std::pin::Pin<
                ::std::boxed::Box<
                    dyn ::std::future::Future<
                        Output = ::std::result::Result<::serde_json::Value, ::std::string::String>,
                    > + ::std::marker::Send + '_,
                >,
            > {
                ::std::boxed::Box::pin(async move {
                    #(#bindings)*
                    #produce
                    ::serde_json::to_value(value)
                        .map_err(|e| format!("{}: result did not serialize: {}", #name_str, e))
                })
            }

            ::inventory::submit! {
                crate::rpc::Entry { name: #name_str, call }
            }
        }
    })
}

fn classify(ty: &Type, pat: &Pat) -> Result<Source, String> {
    if let Some(last) = last_segment(ty) {
        match last.0.as_str() {
            "State" => {
                // `State<'_, T>` — the lifetime is first, the type second.
                let PathArguments::AngleBracketed(generics) = &last.1 else {
                    return Err("State must be written `State<'_, T>`".into());
                };
                let inner = generics.args.iter().find_map(|a| match a {
                    syn::GenericArgument::Type(t) => Some(t.clone()),
                    _ => None,
                });
                return inner
                    .map(|t| Source::State(Box::new(t)))
                    .ok_or_else(|| "State must be written `State<'_, T>`".into());
            }
            "AppHandle" => return Ok(Source::Handle),
            "Window" | "WebviewWindow" => {
                return Err(
                    "this command needs a window, so it cannot be served over /rpc — mark it \
                     `#[rpc(local_only)]` to keep it IPC-only"
                        .into(),
                )
            }
            _ => {}
        }
    }

    // Caught here so the author gets this sentence instead of serde's
    // "implementation of `Deserialize` is not general enough", which does not
    // point at the parameter or say what to do about it.
    if matches!(ty, Type::Reference(_)) {
        return Err(
            "a borrowed parameter cannot be deserialised from a request — take an owned \
             `String` (or mark the command `#[rpc(local_only)]`)"
                .into(),
        );
    }

    let Pat::Ident(ident) = pat else {
        return Err("#[rpc] needs a plain parameter name to read the argument from".into());
    };
    let snake = ident.ident.to_string();
    let camel = to_camel_case(&snake);
    Ok(Source::Arg { snake, camel })
}

/// Peel references and return the final path segment's name and generics.
fn last_segment(ty: &Type) -> Option<(String, PathArguments)> {
    match ty {
        Type::Reference(r) => last_segment(&r.elem),
        Type::Path(p) => p
            .path
            .segments
            .last()
            .map(|s| (s.ident.to_string(), s.arguments.clone())),
        _ => None,
    }
}

fn returns_result(output: &ReturnType) -> bool {
    let ReturnType::Type(_, ty) = output else {
        return false;
    };
    matches!(last_segment(ty), Some((name, _)) if name == "Result")
}

/// `thread_id` -> `threadId`. Tauri maps camelCase from JS onto snake_case
/// params, so the daemon accepts the same spelling a frontend already sends.
fn to_camel_case(snake: &str) -> String {
    let mut out = String::with_capacity(snake.len());
    let mut upper_next = false;
    for ch in snake.chars() {
        if ch == '_' {
            upper_next = true;
        } else if upper_next {
            out.extend(ch.to_uppercase());
            upper_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn span_of(arg: &FnArg) -> proc_macro2::Span {
    use syn::spanned::Spanned;
    arg.span()
}

fn compile_error(span: proc_macro2::Span, message: &str) -> TokenStream {
    syn::Error::new(span, message).to_compile_error().into()
}

#[cfg(test)]
mod tests {
    use super::to_camel_case;

    #[test]
    fn camel_case_matches_what_the_frontend_sends() {
        assert_eq!(to_camel_case("thread_id"), "threadId");
        assert_eq!(to_camel_case("workspace_root"), "workspaceRoot");
        assert_eq!(to_camel_case("model"), "model");
        assert_eq!(to_camel_case("a_b_c"), "aBC");
    }
}
