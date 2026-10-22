//! Circle-bound, structured profiles and their executable recognition boundary.
use crate::program::{self, Change};
use watad_checker::{obligations, poem, profile as checked, structured, verse};
use watad_daira::MeterName;
use watad_types::Beat;

pub use watad_checker::verse::{Certificate, DarbClass, ProfileId, Scope};

pub use watad_checker::catalog::Form;

pub struct Definition {
    pub id: ProfileId,
    pub meter: MeterName,
    pub source: &'static str,
    pub binding: verse::BindingPolicy,
    pub forms: Vec<Form>,
    /// Every group requires at least one selected branch somewhere in a poem.
    pub requirements: Vec<Vec<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("missing source provenance")]
    MissingSource,
    #[error("layout must retain a nonempty prefix of the meter's derived feet")]
    Layout,
    #[error("invalid transformation recipe: {0}")]
    Recipe(#[from] program::Error),
    #[error("invalid structured profile")]
    Profile,
    #[error("a poem requirement must name existing branches and be nonempty")]
    Requirement,
    #[error("poem input is empty")]
    EmptyPoem,
}

/// Both the source definition and compiled authority are immutable after
/// validation. Certificates never supply or override either authority.
pub struct MeterProfile {
    definition: Definition,
    core: checked::Profile,
}

pub(crate) fn lower_slots(
    meter: MeterName,
    recipes: &[Vec<Vec<Change>>],
) -> Result<Vec<structured::FootSlot>, Error> {
    watad_checker::catalog::lower_slots(meter.feet(), recipes).map_err(|error| match error {
        watad_checker::catalog::Error::Recipe(error) => Error::Recipe(error),
        _ => Error::Layout,
    })
}

impl MeterProfile {
    pub fn new(definition: Definition) -> Result<Self, Error> {
        if definition.source.trim().is_empty() {
            return Err(Error::MissingSource);
        }
        let mut forms = Vec::new();
        for form in &definition.forms {
            if form.feet == 0
                || form.feet > definition.meter.feet().len()
                || form.sadr.len() != form.feet
                || (!form.ajuz.is_empty() && form.ajuz.len() != form.feet)
            {
                return Err(Error::Layout);
            }
            forms.push(structured::Form {
                sadr_constraints: form.constraints.clone(),
                ajuz_constraints: if form.ajuz.is_empty() {
                    vec![]
                } else {
                    form.constraints.clone()
                },
                context: form.context,
                initial_only: form.initial_only,
                class: form.class,
                sadr: lower_slots(definition.meter, &form.sadr)?,
                ajuz: lower_slots(definition.meter, &form.ajuz)?,
            });
        }
        if definition
            .requirements
            .iter()
            .any(|group| group.is_empty() || group.iter().any(|i| *i >= forms.len()))
        {
            return Err(Error::Requirement);
        }
        let structured = structured::Profile {
            id: definition.id,
            binding: definition.binding,
            forms,
        };
        let requirements = obligations::Requirements {
            groups: definition.requirements.clone(),
        };
        let meter = definition.meter;
        let core = checked::Profile::new_bound(
            structured,
            requirements,
            meter.circle().id(),
            meter.rotation(),
            meter.circle().reps_per_hemistich(),
            meter.feet().to_vec(),
        )
        .map_err(|_| Error::Profile)?;
        Ok(Self { definition, core })
    }

    pub fn definition(&self) -> &Definition {
        &self.definition
    }
    pub fn is_circle_bound(&self) -> bool {
        self.core.is_circle_bound()
    }
    pub fn structured(&self) -> &structured::Profile {
        self.core.source()
    }
    pub fn kernel(&self) -> &verse::Profile {
        self.core.kernel()
    }

    /// Exact verdict for the externally selected profile and context.
    pub fn recognize(
        &self,
        scope: Scope,
        sadr: &[Beat],
        ajuz: &[Beat],
    ) -> Result<Option<Certificate>, Error> {
        self.core
            .recognize(scope, sadr, ajuz)
            .map_err(|_| Error::Profile)
    }

    pub fn check(&self, scope: Scope, sadr: &[Beat], ajuz: &[Beat], c: &Certificate) -> bool {
        self.core.check(scope, sadr, ajuz, c)
    }

    pub fn recognize_poem(&self, input: &[poem::Input]) -> Result<Option<Vec<Certificate>>, Error> {
        self.core.recognize_poem(input).map_err(|e| match e {
            poem::Error::InvalidProfile => Error::Profile,
            poem::Error::EmptyPoem => Error::EmptyPoem,
        })
    }

    pub fn check_poem(&self, input: &[poem::Input], certificates: &[Certificate]) -> bool {
        self.core.check_poem(input, certificates)
    }
}
