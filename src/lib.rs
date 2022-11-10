mod dlt_layer;

pub use dlt_layer::DltLayer;

#[cfg(test)]
mod tests {
    use tracing::info;
    use tracing_subscriber::prelude::*;
    use super::*;
    use dlt_layer::DltLayer;
    use tracing::{info_span, debug_span, error_span};

    #[test]
    fn it_works() {
       
        tracing_subscriber::registry().with(DltLayer::new("APP","An example application")).init();
        
        let outer_span = info_span!("outer", level = 0);
        let _outer_entered = outer_span.enter();
    
        let inner_span = error_span!("inner", level = 1);
        let _inner_entered = inner_span.enter();
    
        info!(a_bool = true, answer = 42, message = "first example");
    }
}
