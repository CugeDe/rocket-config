use inflector::Inflector as _;
use proc_macro::TokenStream;
use syn::Result;
use syn::parse::{Parse, ParseStream};

#[derive(Debug)]
struct ConfigurationInput {
    /// The name of the structure to be generated.
    type_name: proc_macro2::Ident,

    /// The file stem as passed in via `configuration!("configuration file stem")`.
    file_stem: String,
}

impl Parse for ConfigurationInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let head = input.fork();

        let stem = match input.parse()? {
            syn::Lit::Str(lit) => lit.value(),
            _ => return Err(head.error("expected string literal"))
        };

        let type_name = (stem.clone() + "Configuration").to_pascal_case();

        Ok(Self {
            file_stem: stem,
            type_name: format_ident!("{}", type_name)
        })
    }
}

#[allow(non_snake_case)]
pub fn configuration_function(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ConfigurationInput);

    // Store everything we're going to need to generate code.
    let configuration_stem = &input.file_stem;
    let configuration_type = &input.type_name;

    // A few useful paths.
    let configuration = quote!(::rocket_config::Configuration);
    let error = quote!(::rocket_config::error);
    let factory = quote!(::rocket_config::Factory);
    let index = quote!(::rocket_config::Index);
    let outcome = quote!(::rocket::outcome::Outcome);
    let request = quote!(::rocket::request);
    let result = quote!(::rocket_config::Result);
    let state = quote!(::rocket::State);
    let status = quote!(::rocket::http::Status);
    let value = quote!(::rocket_config::Value);

    let generated_type = quote! {
        /// The request guard type.
        #[derive(Debug)]
        pub struct #configuration_type(#configuration);
    };

    let impl_generated_type = quote! {
        impl #configuration_type {
            #[allow(dead_code)]
            pub fn get<I: #index>(&self, index: I) -> #result<Option<#value>>
            {
                self.0.get(index)
            }
        }
    };

    let impl_from_request = quote! {
        impl<'a, 'r> #request::FromRequest<'a, 'r> for #configuration_type {
            type Error = #error::Error;

            fn from_request(request: &'a #request::Request<'r>) -> #request::Outcome<Self, Self::Error>
            {
                match request.guard::<#state<#factory>>() {
                    #outcome::Success(factory)   => {
                        match factory.get(#configuration_stem) {
                            Ok(config)          => #outcome::Success(Self(config)),
                            Err(err)            => {
                                #outcome::Failure((
                                    #status::InternalServerError,
                                    err
                                ))
                            }
                        }
                    },
                    #outcome::Failure(_failure)  => {
                        #outcome::Failure((
                            #status::InternalServerError,
                            Self::Error::new(
                                #error::ErrorKind::Other,
                                ("failed to get".to_owned() + &#configuration_stem).to_owned() + "configuration"
                            )
                        ))
                    }
                    #outcome::Forward(_)         => { unreachable!() },
                }
            }
        }
    };

    (quote! {
        #generated_type
        #impl_generated_type
        #impl_from_request
    }).into()
}