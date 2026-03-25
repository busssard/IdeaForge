use aiken/transaction.{ScriptContext, Spend}
use aiken/transaction/value
use aiken/interval.{is_entirely_before, is_entirely_after}
use aiken/list
use aiken/transaction/credential.{VerificationKeyCredential}
use ideaforge/types.{PledgeDatum, PledgeAction}

/// Parameterized escrow validator
/// platform_key: The IdeaForge platform's verification key hash (multisig co-signer)
validator(platform_key: VerificationKeyHash) {
  fn pledge_escrow(datum: PledgeDatum, redeemer: PledgeAction, ctx: ScriptContext) -> Bool {
    let ScriptContext { transaction, purpose } = ctx

    expect Spend(_) = purpose

    let valid_signatories = transaction.extra_signatories

    when redeemer is {
      // Creator claims: requires creator sig + platform co-sig + before deadline + target met
      Claim -> {
        let creator_signed = list.has(valid_signatories, datum.creator)
        let platform_signed = list.has(valid_signatories, platform_key)
        let before_deadline = is_entirely_before(transaction.validity_range, datum.deadline)

        // Check total value at script address meets target
        // Aggregate all input values to verify the campaign total
        let total_input_value =
          list.foldl(
            transaction.inputs,
            value.zero(),
            fn(input, acc) { value.merge(acc, input.output.value) },
          )
        let script_lovelace = value.lovelace_of(total_input_value)
        let target_met = script_lovelace >= datum.min_target

        creator_signed && platform_signed && before_deadline && target_met
      }

      // Pledger refund: requires pledger sig + after deadline
      Refund -> {
        let pledger_signed = list.has(valid_signatories, datum.pledger)
        let after_deadline = is_entirely_after(transaction.validity_range, datum.deadline)

        pledger_signed && after_deadline
      }

      // Platform refund: requires platform sig + funds go back to pledger
      PlatformRefund -> {
        let platform_signed = list.has(valid_signatories, platform_key)

        // Verify at least one output pays to the pledger's address
        let pays_pledger =
          list.any(
            transaction.outputs,
            fn(output) {
              when output.address.payment_credential is {
                VerificationKeyCredential(hash) -> hash == datum.pledger
                _ -> False
              }
            },
          )

        platform_signed && pays_pledger
      }
    }
  }
}
