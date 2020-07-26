use super::{
    common::{create_attestation, validate_last_header_still_unchanged},
    complete_transaction::CompleteTransactionRequest,
};
use crate::{
    message,
    message::{MessageBody, OfferMessage, OfferResponse},
    offer,
    offer::OfferState,
    transaction, utils,
};
use hdk::prelude::*;
use holochain_entry_utils::HolochainEntry;
use transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, self::DefaultJson, Clone)]
pub struct AcceptOfferRequest {
    transaction_address: Address,
    approved_header_address: Address,
}

/**
 * Accepts the offer, verifying that the source chain of the sender agent has not changed,
 * and creating the transaction privately
 */
pub fn accept_offer(
    transaction_address: Address,
    approved_header_address: Address,
) -> ZomeApiResult<()> {
    let offer = offer::query_offer(&transaction_address)?;

    let transaction = offer.transaction;

    match offer.state {
        OfferState::Pending => {
            offer::approve_offer(&transaction_address, &Some(approved_header_address.clone()))?;

            let response =
                send_accept_offer(&transaction_address, &approved_header_address, &transaction);

            match response {
                Ok(OfferResponse::OfferCompleted(attestation_address)) => {
                    offer::complete_offer(&transaction_address, &attestation_address)?;
                    Ok(())
                }
                Ok(OfferResponse::OfferCanceled) => {
                    offer::cancel_offer(&transaction_address)?;
                    Err(ZomeApiError::from(format!("Offer was canceled")))
                }
                Ok(OfferResponse::OfferPending(())) => {
                    offer::update_offer_state(&transaction_address, OfferState::Pending)?;
                    Err(ZomeApiError::from(String::from("Offer is still pending")))
                }
                Err(err) => {
                    offer::update_offer_state(&transaction_address, OfferState::Pending)?;
                    Err(err)
                }
            }
        }
        _ => Err(ZomeApiError::from(String::from(
            "Offer is not pending, it cannot be accepted",
        ))),
    }
}

/**
 * Verifies that offer is approved, and then begins the complete transaction workflow
 */
pub fn receive_accept_offer(
    sender_address: Address,
    accept_offer_request: AcceptOfferRequest,
) -> ZomeApiResult<OfferResponse<()>> {
    let offer = offer::query_offer(&accept_offer_request.transaction_address)?;

    let transaction = offer.transaction;

    let counterparty = transaction::get_counterparty(&transaction);

    if sender_address != counterparty {
        return Err(ZomeApiError::from(String::from(
            "Agent sending the message is not the counterparty for this transaction",
        )));
    }

    match offer.state {
        OfferState::Approved {
            approved_header_address,
        } => {
            let attestation_address =
                handle_accept_offer(accept_offer_request, approved_header_address)?;

            Ok(OfferResponse::OfferCompleted(attestation_address))
        }
        OfferState::Canceled => Ok(OfferResponse::OfferCanceled),
        _ => Err(ZomeApiError::from(format!(
            "Offer for transaction {} has not been approved",
            accept_offer_request.transaction_address
        ))),
    }
}

/** Private helpers **/

fn send_accept_offer(
    transaction_address: &Address,
    approved_header_address: &Address,
    transaction: &Transaction,
) -> ZomeApiResult<OfferResponse<()>> {
    let accept_offer_request = AcceptOfferRequest {
        transaction_address: transaction_address.clone(),
        approved_header_address: approved_header_address.clone(),
    };

    let message = MessageBody::AcceptOffer(OfferMessage::Request(accept_offer_request));

    // TODO: generalize transaction.debtor_address to be the counterparty
    let result = message::send_message(transaction.debtor_address.clone(), message)?;

    match result {
        MessageBody::AcceptOffer(OfferMessage::Response(response)) => Ok(response),
        _ => Err(ZomeApiError::from(format!(
            "CompleteOffer response is not valid"
        ))),
    }
}

/**
 * Handles an incoming AcceptOfferRequest, assuming that the offer was approved
 *
 * 1. Check that the approved_header_address is still the same
 * 2. Create the transaction
 * 3. Get the transaction header
 * 4. Send a CompleteTransactionRequest
 * 5. Create Attestation
 */
fn handle_accept_offer(
    accept_offer_request: AcceptOfferRequest,
    _approved_header_address: Option<Address>, // TODO: in the future, verify that creditor hasn't also committed anything new
) -> ZomeApiResult<Address> {
    validate_last_header_still_unchanged(accept_offer_request.approved_header_address)?;

    let offer = offer::query_offer(&accept_offer_request.transaction_address)?;
    hdk::commit_entry(&offer.transaction.clone().entry())?;

    let transaction_header = utils::get_my_last_header()?;

    let complete_transaction_request = CompleteTransactionRequest {
        chain_header: transaction_header,
    };

    let message =
        MessageBody::CompleteTransaction(OfferMessage::Request(complete_transaction_request));

    let counterparty_address = transaction::get_counterparty(&offer.transaction);

    let result = message::send_message(counterparty_address, message)?;

    match result {
        MessageBody::CompleteTransaction(OfferMessage::Response(OfferResponse::OfferPending(
            complete_transaction_response,
        ))) => {
            let attestation_address = create_attestation(
                &complete_transaction_response.chain_headers,
                &complete_transaction_response.signature,
            )?;

            hdk::emit_signal(
                "offer-completed",
                JsonString::from_json(&format!(
                    "{{\"transaction_address\": \"{}\"}}",
                    offer.transaction.address()?
                )),
            )?;

            Ok(attestation_address)
        }
        MessageBody::CompleteTransaction(OfferMessage::Response(OfferResponse::OfferCanceled)) => {
            offer::cancel_offer(&accept_offer_request.transaction_address)?;
            Err(ZomeApiError::from(format!("Offer was canceled")))
        }
        _ => Err(ZomeApiError::from(format!(
            "Received error when attempting to complete the transaction {:?}",
            result
        ))),
    }
}
