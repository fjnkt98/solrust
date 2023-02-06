use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn impl_common_query_parser(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input.into()).expect("Failed to parse input TokenStream");
    let struct_name = ast.ident;
    let gen = quote::quote! {
        impl SolrCommonQueryBuilder for #struct_name {
            fn sort(mut self, sort: &SortOrderBuilder) -> Self {
                self.params.insert("sort".to_string(), sort.build());
                self
            }

            fn start(mut self, start: u32) -> Self {
                self.params.insert("start".to_string(), start.to_string());
                self
            }

            fn rows(mut self, rows: u32) -> Self {
                self.params.insert("rows".to_string(), rows.to_string());
                self
            }

            fn fq(mut self, fq: &impl SolrQueryExpression) -> Self {
                self.multi_params
                    .entry("fq".to_string())
                    .or_default()
                    .push(fq.to_string());
                self
            }

            fn fl(mut self, fl: String) -> Self {
                self.params.insert("fl".to_string(), fl);
                self
            }

            fn debug(mut self) -> Self {
                self.params.insert("debug".to_string(), "all".to_string());
                self.params
                    .insert("debug.explain.structured".to_string(), "true".to_string());
                self
            }
            fn wt(mut self, wt: &str) -> Self {
                self.params.insert("wt".to_string(), wt.to_string());
                self
            }
            fn facet(mut self, facet: &impl FacetBuilder) -> Self {
                self.params.insert("facet".to_string(), "true".to_string());
                for (key, value) in facet.build() {
                    self.params.insert(key, value);
                }
                self
            }

            fn op(mut self, op: Operator) -> Self {
                match op {
                    Operator::AND => {
                        self.params.insert("q.op".to_string(), "AND".to_string());
                    }
                    Operator::OR => {
                        self.params.insert("q.op".to_string(), "OR".to_string());
                    }
                }
                self
            }

            fn build(self) -> Vec<(String, String)> {
                let mut params = Vec::new();

                params.extend(self.params.into_iter());
                for (key, values) in self.multi_params.into_iter() {
                    params.extend(values.into_iter().map(|param| (key.clone(), param)));
                }

                params
            }
        }
    };
    gen.into()
}

pub fn impl_dismax_query_parser(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input.into()).expect("Failed to parse input TokenStream");
    let struct_name = ast.ident;
    let gen = quote::quote! {
        impl SolrDisMaxQueryBuilder for #struct_name {
            fn q(mut self, q: String) -> Self {
                // TODO: 引数の型を抽象モデルに変更する
                self.params.insert("q".to_string(), q.to_string());
                self
            }
            fn qf(mut self, qf: &str) -> Self {
                self.params.insert("qf".to_string(), qf.to_string());
                self
            }
            fn qs(mut self, qs: u32) -> Self {
                self.params.insert("qs".to_string(), qs.to_string());
                self
            }
            fn pf(mut self, pf: &str) -> Self {
                self.params.insert("pf".to_string(), pf.to_string());
                self
            }
            fn ps(mut self, ps: u32) -> Self {
                self.params.insert("ps".to_string(), ps.to_string());
                self
            }
            fn mm(mut self, mm: &str) -> Self {
                self.params.insert("mm".to_string(), mm.to_string());
                self
            }
            fn q_alt(mut self, q: &impl SolrQueryExpression) -> Self {
                self.params.insert("q.alt".to_string(), q.to_string());
                self
            }
            fn tie(mut self, tie: f64) -> Self {
                self.params.insert("tie".to_string(), tie.to_string());
                self
            }
            fn bq(mut self, bq: &impl SolrQueryExpression) -> Self {
                self.multi_params
                    .entry("bq".to_string())
                    .or_default()
                    .push(bq.to_string());
                self
            }
            fn bf(mut self, bf: &str) -> Self {
                self.multi_params
                    .entry("bf".to_string())
                    .or_default()
                    .push(bf.to_string());
                self
            }
        }
    };
    gen.into()
}
