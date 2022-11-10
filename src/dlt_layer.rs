use std::ffi::{CString, CStr};

use autoincrement::{AsyncIncremental, AsyncIncrement};
use tracing::Level;
use tracing_subscriber::{Layer, registry::LookupSpan};
use libdlt_sys::{self, DltContext, dlt_register_context, DltLogLevelType, dlt_user_log_write_start_id, DltContextData, DltReturnValue, dlt_user_log_write_string, dlt_user_log_write_bool, dlt_user_log_write_uint64, dlt_user_log_write_int64, dlt_user_log_write_float64, dlt_user_log_write_finish, dlt_register_app,};

#[derive(AsyncIncremental, PartialEq, Eq, Debug)]
struct ContextId(u16);
pub struct DltLayer(AsyncIncrement<ContextId>);

impl DltLayer {
    pub fn new(app_id:&str, description:&str) -> Self {
        let app_id = CString::new(app_id).unwrap();
        unsafe{dlt_register_app(app_id.as_ptr(),CString::new(description).unwrap().as_ptr())};
        Self(ContextId::init())

    }
}

#[derive(Debug)]
struct CustomFieldStorage(libdlt_sys::DltContext);

// Safety:
// DltContext is thread safe according
// to the Dlt documentaion.
unsafe impl Send for CustomFieldStorage {}
unsafe impl Sync for CustomFieldStorage {}

impl<S> Layer<S> for DltLayer where S : tracing::Subscriber +  for<'a> LookupSpan<'a> {

    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut scope = ctx.event_scope(event).unwrap();
        
        if let Some(span) = scope.next() {
            let mut extensions = span.extensions_mut();
            let storage = extensions.get_mut::<CustomFieldStorage>().unwrap();
            if let Some(mut visitor) = DltVisitor::new(*span.metadata().level(),&mut storage.0) {
                event.record(&mut visitor);
            }
        }
    }

    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
            
        // Get a reference to the internal span data
        let span = ctx.span(id).unwrap();
        
        // Get the special place where tracing stores custom data
        let mut extensions = span.extensions_mut();

        // Create a new DltContext
        let mut dlt_context = DltContext::new_uninitialized();
        let context_id = CString::new(format!("{}",self.0.pull().0)).unwrap();
        let description = CString::new(span.metadata().name()).unwrap();
        let storage = unsafe{
            dlt_register_context(dlt_context.as_mut_ptr(), context_id.as_ptr(), description.as_ptr());
            CustomFieldStorage(dlt_context.assume_init())
        };

        // And store our data
        extensions.insert::<CustomFieldStorage>(storage);
    }
    
}


struct DltVisitor(DltContextData);

impl DltVisitor {

    fn new(level: Level, context: & mut libdlt_sys::DltContext ) -> Option<Self> {
        let mut local_context = DltContextData::new_uninitialized();
        unsafe {
            if dlt_user_log_write_start_id(
                context,
                local_context.as_mut_ptr(),
                Self::get_level(level),
                0,
            ) == DltReturnValue::DLT_RETURN_TRUE  {
                Some(DltVisitor(local_context.assume_init()))
            } else {
                None
            }
        }
    }

    fn get_level(level: Level) -> DltLogLevelType {
        match level {
            Level::DEBUG => DltLogLevelType::DLT_LOG_DEBUG,
            Level::ERROR => DltLogLevelType::DLT_LOG_ERROR,
            Level::INFO => DltLogLevelType::DLT_LOG_INFO,
            Level::TRACE => DltLogLevelType::DLT_LOG_VERBOSE,
            Level::WARN => DltLogLevelType::DLT_LOG_WARN,
        }
    }
}

impl Drop for DltVisitor {
    fn drop(&mut self) {
        unsafe {
            dlt_user_log_write_finish(&mut self.0); 
        }
    }
}

impl tracing::field::Visit for DltVisitor {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        //println!("  field={} value={}", field.name(), &value);
        let name=CString::new(field.name()).unwrap();
        unsafe {
            let eq = CStr::from_bytes_with_nul_unchecked(b"=\0");
            dlt_user_log_write_string(&mut self.0, name.as_ptr());
            dlt_user_log_write_string(&mut self.0, eq.as_ptr());
            dlt_user_log_write_float64(&mut self.0, value);
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        //println!("  field={} value={}", field.name(), &value);
        let name=CString::new(field.name()).unwrap();
        unsafe {
            let eq = CStr::from_bytes_with_nul_unchecked(b"=\0");
            dlt_user_log_write_string(&mut self.0, name.as_ptr());
            dlt_user_log_write_string(&mut self.0, eq.as_ptr());
            dlt_user_log_write_int64(&mut self.0, value);
        }
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        //println!("  field={} value={}", field.name(), &value);
        let name=CString::new(field.name()).unwrap();
        unsafe {
            let eq = CStr::from_bytes_with_nul_unchecked(b"=\0");
            dlt_user_log_write_string(&mut self.0, name.as_ptr());
            dlt_user_log_write_string(&mut self.0, eq.as_ptr());
            dlt_user_log_write_uint64(&mut self.0, value);
        }
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        //println!("  field={} value={}", field.name(), &value);
        let name=CString::new(field.name()).unwrap();
        unsafe {
            let eq = CStr::from_bytes_with_nul_unchecked(b"=\0");
            dlt_user_log_write_string(&mut self.0, name.as_ptr());
            dlt_user_log_write_string(&mut self.0, eq.as_ptr());
            dlt_user_log_write_bool(&mut self.0, value as u8);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
      
        let s = CString::new(value).unwrap();
        let name=CString::new(field.name()).unwrap();
        
        unsafe {
            let eq = CStr::from_bytes_with_nul_unchecked(b"=\0");
            dlt_user_log_write_string(&mut self.0, name.as_ptr());
            dlt_user_log_write_string(&mut self.0, eq.as_ptr());
            dlt_user_log_write_string(&mut self.0, s.as_ptr());
        }

    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        //println!("  field={} value={:?}", field.name(), &value);
        let name=CString::new(field.name()).unwrap();
        let text = CString::new(format!("{:?}",value)).unwrap();
        unsafe {
            let eq = CStr::from_bytes_with_nul_unchecked(b"=\0");
            dlt_user_log_write_string(&mut self.0, name.as_ptr());
            dlt_user_log_write_string(&mut self.0, eq.as_ptr());
            dlt_user_log_write_string(&mut self.0, text.as_ptr());

        }
    }
}