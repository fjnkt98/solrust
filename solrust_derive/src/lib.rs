use proc_macro::TokenStream;

#[proc_macro_derive(SolrCommonQueryParser)]
pub fn derive_common_query_parser(input: TokenStream) -> TokenStream {
    solrust_derive_internals::impl_common_query_parser(input.into()).into()
}

#[proc_macro_derive(SolrStandardQueryParser)]
pub fn derive_standard_query_parser(input: TokenStream) -> TokenStream {
    solrust_derive_internals::impl_standard_query_parser(input.into()).into()
}
#[proc_macro_derive(SolrDisMaxQueryParser)]
pub fn derive_dismax_query_parser(input: TokenStream) -> TokenStream {
    solrust_derive_internals::impl_dismax_query_parser(input.into()).into()
}

#[proc_macro_derive(SolrEDisMaxQueryParser)]
pub fn derive_edismax_query_parser(input: TokenStream) -> TokenStream {
    solrust_derive_internals::impl_edismax_query_parser(input.into()).into()
}
