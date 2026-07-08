//! AI-domain-backed verification of cleanup proposals.
//!
//! Replaces the standalone Agent Platform round-trip (`verify.rs`) with a single
//! call through the desktop's agent-platform provider. Same contract as before:
//! reasons from metadata, never deletes or lowers risk. The prompt + verdict
//! parsing are reused verbatim from `verify.rs` so the UI is unchanged.

use crate::domains::ai::providers::GenerationOptions;
use crate::domains::ai::services::AIService;
use crate::domains::disk::classify::Proposal;
use crate::domains::disk::verify::{build_goal, parse_verdicts, AgentNote, VerificationResult};

/// System persona that mirrors the old multi-agent roster's stance in one voice.
const SYSTEM: &str = "You are a cautious disk-cleanup reviewer. You judge whether deletion \
candidates (moved to the Recycle Bin, reversible) are safe, need review, or are dangerous. \
Reason ONLY from the metadata you are given — never invent files or assume contents. Flag \
anything that could be irreplaceable user data or hard to regenerate; bias toward 'review' when \
uncertain. You are advisory only: recommend, never instruct deletion, and never lower a risk.";

/// Verify proposals via the configured AI provider (default provider if `None`).
/// Single-shot generation; the model's full reply is returned as one agent note
/// and per-item verdicts are parsed from its trailing JSON block.
pub async fn verify_with_ai(
    root: String,
    proposals: Vec<Proposal>,
    ai: &AIService,
) -> Result<VerificationResult, String> {
    if proposals.is_empty() {
        return Err("Nothing to verify.".to_string());
    }

    let user = build_goal(&root, &proposals);
    let options = GenerationOptions {
        // Deterministic-ish review; give room for the per-item JSON list.
        temperature: Some(0.2),
        max_tokens: Some(4096),
        ..Default::default()
    };

    let result = ai
        .generate_with_system(SYSTEM, &user, Some(options), None)
        .await
        .map_err(|e| e.to_string())?;

    let note = AgentNote {
        task_id: 0,
        role: format!("AI Reviewer ({})", result.model),
        status: "completed".to_string(),
        output: result.content.clone(),
    };
    let verdicts = parse_verdicts(std::slice::from_ref(&note));

    Ok(VerificationResult {
        process_id: 0,
        status: "completed".to_string(),
        notes: vec![note],
        verdicts,
        gated: false,
    })
}
