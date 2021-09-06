use syn::{
    parse::{Parse, ParseStream, Result},
    Ident,
};

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Events(pub Vec<Event>);

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Event {
    pub name: Ident,
}

impl Parse for Event {
    /// example event tokens:
    ///
    /// ```text
    /// Push
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name = input.parse()?;

        Ok(Event { name })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::{self, parse_quote};

    #[test]
    fn test_event_parse() {
        let left: Event = syn::parse2(quote! { Push }).unwrap();
        let right = Event {
            name: parse_quote! { Push },
        };

        assert_eq!(left, right);
    }
}
