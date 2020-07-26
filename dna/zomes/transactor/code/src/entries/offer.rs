use crate::transaction::Transaction;
use crate::utils;
use hdk::entry_definition::ValidatingEntryType;
use hdk::holochain_core_types::chain_header::ChainHeader;
use hdk::holochain_json_api::{error::JsonError, json::JsonString};
use hdk::holochain_persistence_api::cas::content::Address;
use hdk::{
    error::{ZomeApiError, ZomeApiResult},
    holochain_core_types::dna::entry_types::Sharing,
};
use holochain_entry_utils::HolochainEntry;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub enum OfferState {
    Received,
    Pending,
    Canceled,
    Approved {
        approved_header_address: Option<Address>,
    },
    Completed {
        attestation_address: Address,
    },
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Offer {
    pub transaction: Transaction,
    pub state: OfferState,
}

impl HolochainEntry for Offer {
    fn entry_type() -> String {
        String::from("offer")
    }
}

pub fn entry_definition() -> ValidatingEntryType {
    entry!(
        name: Offer::entry_type(),
        description: "offer private entry to temporarily store the data of a transaction before accepting it",
        sharing: Sharing::Private,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |_validation_data: hdk::EntryValidationData<Offer>| {
            Ok(())
        }
    )
}

/**
 * Query all offers present in our source chain, getting only the last offer for each transaction to maintain state consistency
 */
pub fn query_my_offers() -> ZomeApiResult<Vec<(Address, Offer)>> {
    let offers: Vec<(ChainHeader, Offer)> = utils::query_all_into()?;

    let mut transaction_map: HashMap<Address, Offer> = HashMap::new();

    for offer in offers {
        let transaction_address = offer.1.transaction.address()?;
        if !transaction_map.contains_key(&transaction_address) {
            transaction_map.insert(transaction_address, offer.1);
        }
    }

    Ok(transaction_map.into_iter().collect())
}

/**
 * Gets the last offer identified with the given address from the private chain
 */
pub fn query_offer(transaction_address: &Address) -> ZomeApiResult<Offer> {
    let offers: Vec<(ChainHeader, Offer)> = utils::query_all_into()?;

    let maybe_offer =
        offers
            .iter()
            .map(|next_offer| next_offer.1.clone())
            .find(|offer| match offer.transaction.address() {
                Ok(address) => address == transaction_address.clone(),
                Err(_) => false,
            });

    maybe_offer.ok_or(ZomeApiError::from(format!(
        "Could not find offer for transaction address {}",
        transaction_address
    )))
}

/**
 * Updates the private offer to a canceled state
 */
pub fn cancel_offer(transaction_address: &Address) -> ZomeApiResult<()> {
    update_offer_state(transaction_address, OfferState::Canceled)
}

/**
 * Updates the private offer to a completed state
 */
pub fn approve_offer(
    transaction_address: &Address,
    approved_header_address: &Option<Address>,
) -> ZomeApiResult<()> {
    update_offer_state(
        transaction_address,
        OfferState::Approved {
            approved_header_address: approved_header_address.clone(),
        },
    )
}

/**
 * Updates the private offer to a completed state
 */
pub fn complete_offer(
    transaction_address: &Address,
    attestation_address: &Address,
) -> ZomeApiResult<()> {
    update_offer_state(
        transaction_address,
        OfferState::Completed {
            attestation_address: attestation_address.clone(),
        },
    )
}

/**
 * Updates the private offer to the given offer state
 */
pub fn update_offer_state(transaction_address: &Address, offer_state: OfferState) -> ZomeApiResult<()> {
    let mut offer = query_offer(transaction_address)?;

    let current_address = offer.address()?;
    offer.state = offer_state;

    hdk::update_entry(offer.entry(), &current_address)?;

    Ok(())
}
