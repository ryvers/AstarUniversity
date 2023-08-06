use crate::Governor::*;
/// A helper function used for calling contract messages.
use ink_e2e::build_message;

/// The End-to-End test `Result` type.
type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// We test that we can upload and instantiate the contract using its default constructor.
#[ink_e2e::test]
async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    // Given
    Ok(())
}
