/// On-chain datum attached to each pledge UTxO
pub type PledgeDatum {
  /// IdeaForge idea UUID (32 bytes)
  idea_id: ByteArray,
  /// Pledger's payment verification key hash
  pledger: VerificationKeyHash,
  /// Idea creator's payment verification key hash
  creator: VerificationKeyHash,
  /// POSIX timestamp deadline for the pledge campaign
  deadline: Int,
  /// Minimum total pledge target in lovelace
  min_target: Int,
}

/// Actions that can be taken on a pledge UTxO
pub type PledgeAction {
  /// Creator claims funds after successful delivery (requires platform co-sign)
  Claim
  /// Pledger reclaims funds after deadline or target not met
  Refund
  /// Platform-initiated refund (idea abandoned/fraud detected)
  PlatformRefund
}
