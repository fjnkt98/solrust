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
                    // facet.fieldパラメータは複数値を取れるパラメータなので別で処理する
                    if key == "facet.field".to_string() {
                        self.multi_params
                            .entry("facet.field".to_string())
                            .or_default()
                            .push(value);
                    } else {
                        self.params.insert(key, value);
                    }
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

            fn sanitize<'a>(&self, s: &'a str) -> Cow<'a, str> {
                SOLR_SPECIAL_CHARACTERS.replace_all(s, r"\$0")
            }
        }
    };
    gen.into()
}

pub fn impl_standard_query_parser(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input.into()).expect("Failed to parse input TokenStream");
    let struct_name = ast.ident;
    let gen = quote::quote! {
        impl SolrStandardQueryBuilder for #struct_name {
            fn q(mut self, q: &impl SolrQueryExpression) -> Self {
                self.params.insert("q".to_string(), q.to_string());
                self
            }

            fn df(mut self, df: &str) -> Self {
                self.params.insert("df".to_string(), df.to_string());
                self
            }

            fn sow(mut self, sow: bool) -> Self {
                if sow {
                    self.params.insert("sow".to_string(), "true".to_string());
                } else {
                    self.params.insert("sow".to_string(), "false".to_string());
                }
                self
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
                self.params.insert("q".to_string(), self.sanitize(&q).to_string());
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

pub fn impl_edismax_query_parser(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input.into()).expect("Failed to parse input TokenStream");
    let struct_name = ast.ident;
    let gen = quote::quote! {
        impl SolrEDisMaxQueryBuilder for #struct_name {
            fn sow(mut self, sow: bool) -> Self {
                if sow {
                    self.params.insert("sow".to_string(), "true".to_string());
                } else {
                    self.params.insert("sow".to_string(), "false".to_string());
                }
                self
            }

            fn boost(mut self, boost: &str) -> Self {
                self.params.insert("boost".to_string(), boost.to_string());
                self
            }

            fn lowercase_operators(mut self, flag: bool) -> Self {
                if flag {
                    self.params.insert("lowercaseOperators".to_string(), "true".to_string());
                } else {
                    self.params.insert("lowercaseOperators".to_string(), "false".to_string());
                }
                self
            }

            fn pf2(mut self, pf: &str) -> Self {
                self.params.insert("pf2".to_string(), pf.to_string());
                self
            }

            fn ps2(mut self, ps: u32) -> Self {
                self.params.insert("ps2".to_string(), ps.to_string());
                self
            }

            fn pf3(mut self, pf: &str) -> Self {
                self.params.insert("pf3".to_string(), pf.to_string());
                self
            }

            fn ps3(mut self, ps: u32) -> Self {
                self.params.insert("ps3".to_string(), ps.to_string());
                self
            }

            fn stopwords(mut self, flag: bool) -> Self {
                if flag {
                    self.params.insert("stopwords".to_string(), "true".to_string());
                } else {
                    self.params.insert("stopwords".to_string(), "false".to_string());
                }
                self
            }

            fn uf(mut self, uf: &str) -> Self {
                self.params.insert("uf".to_string(), uf.to_string());
                self
            }
        }
    };
    gen.into()
}
