#[derive(serde::Serialize)]
pub struct ValidationErrorsToBeReturned {
    pub errors: Vec<String>,
}
