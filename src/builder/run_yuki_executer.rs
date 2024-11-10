use libcontainer::oci_spec::runtime::Spec;
use libcontainer::workload::{Executor, ExecutorError, ExecutorValidationError};

#[derive(Clone)]
pub struct DefaultExecutor {}

impl Executor for DefaultExecutor {
    fn exec(&self, spec: &Spec) -> Result<(), ExecutorError> {
        // Leave the default executor as the last option, which executes normal
        // container workloads.
        libcontainer::workload::default::get_executor().exec(spec)
    }

    fn validate(&self, spec: &Spec) -> Result<(), ExecutorValidationError> {
        libcontainer::workload::default::get_executor().validate(spec)
    }
}

pub fn default_executor() -> DefaultExecutor {
    DefaultExecutor {}
}
