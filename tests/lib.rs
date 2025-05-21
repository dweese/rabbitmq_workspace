// rabbitmq_workspace/tests/lib.rs

// Define the client test modules
mod client {
    mod unit_tests;
    mod mocked_tests;
    mod integration_tests;
}

// Define the config test modules
mod config {
    mod serialization_tests;
    mod manipulation_tests;
    mod validation_tests;
}

// Define the common utilities module and make it publicly accessible
pub mod common {
    pub mod test_utils;
}