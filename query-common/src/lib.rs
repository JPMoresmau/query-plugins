
#[macro_export]
macro_rules! metadata {
    ( $description:expr, $( $x:expr => $t:ident),* ) => {
        {  
            paste! {
                let mut parameters = Vec::new();
                $(
                    parameters.push(Parameter {
                        name: $x.into(),
                        parameter_type: ParameterType::[<Type $t>],
                       });
                )*
            }
            QueryMetadata {
                description: $description.into(),
                parameters,
            }
        }
    };
}
