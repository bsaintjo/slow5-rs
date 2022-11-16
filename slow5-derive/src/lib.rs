use proc_macro::TokenStream;

#[proc_macro_derive(FieldExt)]
pub fn derive(input: TokenStream) -> TokenStream {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
}
