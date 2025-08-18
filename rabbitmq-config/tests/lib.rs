//  rabbitmq-config/tests/lib.rs

// Define the client test modules
pub mod client {
    pub mod integration_tests;
    pub mod mocked_tests;
    pub mod unit_tests;
}

// Define the config test modules
pub mod config {
    pub mod manipulation_tests;
    pub mod serialization_tests;
    pub mod validation_tests;
}

// Define the common utilities module
pub mod common {
    pub mod test_utils;
}
