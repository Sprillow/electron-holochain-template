use crate::project::error::Error;
use hdk_crud::{resolve_dependency, ResolvedDependency};
use hdk::prelude::*;

pub fn validate_value_matches_create_author(
  value: &AgentPubKey,
  validate_data: &ValidateData,
) -> ValidateCallbackResult {
  match value == validate_data.element.header().author() {
    true => ValidateCallbackResult::Valid,
    false => Error::CorruptCreateAgentPubKeyReference.into(),
  }
}

pub fn validate_value_matches_edit_author(
  value: &AgentPubKey,
  validate_data: &ValidateData,
) -> ValidateCallbackResult {
  match value == validate_data.element.header().author() {
    true => ValidateCallbackResult::Valid,
    false => Error::CorruptEditAgentPubKeyReference.into(),
  }
}

pub fn validate_value_matches_original_author(
  new_value: &AgentPubKey,
  original_value: &AgentPubKey,
) -> ValidateCallbackResult {
  match new_value == original_value {
    true => ValidateCallbackResult::Valid,
    false => Error::TamperCreateAgentPubKeyReference.into(),
  }
}

pub fn validate_value_matches_original_author_for_edit(
  new_value: &AgentPubKey,
  original_value: &AgentPubKey,
) -> ValidateCallbackResult {
  match new_value == original_value {
    true => ValidateCallbackResult::Valid,
    false => Error::UpdateOnNonAuthoredOriginal.into(),
  }
}

pub fn validate_value_is_some<O>(value: &Option<O>) -> ValidateCallbackResult {
  match value {
    Some(_) => ValidateCallbackResult::Valid,
    None => Error::NoneNotSomeDuringEdit.into(),
  }
}

pub fn validate_value_is_none<O>(value: &Option<O>) -> ValidateCallbackResult {
  match value {
    Some(_) => Error::SomeNotNoneDuringCreate.into(),
    None => ValidateCallbackResult::Valid,
  }
}

// should always be called with the HeaderHash of a Create or Update header
pub fn must_get_header_and_entry<O>(header_hash: HeaderHash) -> ExternResult<O>
where
    O: TryFrom<SerializedBytes, Error = SerializedBytesError>,
{
    let h = must_get_header(header_hash)?;
    match h.header().entry_hash() {
        Some(entry_hash) => {
            let entry = must_get_entry(entry_hash.clone())?;
            match entry.into_inner().0 {
                Entry::App(bytes) => match O::try_from(bytes.into()) {
                    Ok(deserialized) => Ok(deserialized),
                    Err(e) => Err(e.into()),
                },
                _ => Err(WasmError::Guest(
                    "entry within resolve_header_to_entry must be an Entry::App variant"
                        .to_string(),
                )),
            }
        }
        None => Err(WasmError::Guest(
            "within resolve_header_to_entry a header that was not a Create variant was attempted"
                .to_string(),
        )),
    }
}

pub fn confirm_resolved_dependency<'a, O>(hash: AnyDhtHash) -> ExternResult<ValidateCallbackResult>
where
  O: TryFrom<SerializedBytes, Error = SerializedBytesError>,
{
  match resolve_dependency::<O>(hash)? {
    Ok(ResolvedDependency(_, _)) => Ok(ValidateCallbackResult::Valid),
    Err(ValidateCallbackResult::Invalid(e)) if e == hdk_crud::Error::EntryMissing.to_string() => {
      // valid because it found the header, and we don't need the entry
      // indicates its a header that really exists
      Ok(ValidateCallbackResult::Valid)
    }
    // we want to forward the validate_callback_result
    // back to Holochain since it contains a specific UnresolvedDependencies response
    // including the missing Hashes
    Err(validate_callback_result) => Ok(validate_callback_result),
  }
}

// if any ValidateCallbackResult is Invalid, then ValidateResult::Invalid
// If none are Invalid and there is an UnresolvedDependencies, then ValidateResult::UnresolvedDependencies
// If all ValidateCallbackResult are Valid, then ValidateResult::Valid
// pub fn reduce_callback_results(
//     callback_results: Vec<ValidateCallbackResult>,
// ) -> ValidateCallbackResult {
//     callback_results
//         .into_iter()
//         .fold(ValidateCallbackResult::Valid, |acc, x| match x {
//             ValidateCallbackResult::Invalid(i) => ValidateCallbackResult::Invalid(i),
//             ValidateCallbackResult::UnresolvedDependencies(ud) => match acc {
//                 ValidateCallbackResult::Invalid(_) => acc,
//                 _ => ValidateCallbackResult::UnresolvedDependencies(ud),
//             },
//             ValidateCallbackResult::Valid => acc,
//         })
// }
