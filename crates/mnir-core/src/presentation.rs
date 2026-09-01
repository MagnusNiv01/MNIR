/// Optional human-oriented data associated with a Module, Function, or Parameter.
///
/// These values never determine Module identity (`MNIR-CORE-022` through
/// `MNIR-CORE-025`).
#[derive(Debug, Default, Eq, PartialEq)]
pub struct PresentationMetadata {
    preferred_name: Option<String>,
    documentation: Option<String>,
}

impl PresentationMetadata {
    #[must_use]
    pub fn preferred_name(&self) -> Option<&str> {
        self.preferred_name.as_deref()
    }

    #[must_use]
    pub fn documentation(&self) -> Option<&str> {
        self.documentation.as_deref()
    }

    pub(crate) fn set_preferred_name(&mut self, preferred_name: Option<String>) {
        self.preferred_name = preferred_name;
    }

    pub(crate) fn set_documentation(&mut self, documentation: Option<String>) {
        self.documentation = documentation;
    }

    pub(crate) fn copied(&self) -> Self {
        Self {
            preferred_name: self.preferred_name.clone(),
            documentation: self.documentation.clone(),
        }
    }
}
