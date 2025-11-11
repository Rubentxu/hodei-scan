# Cap'n Proto schema for Extractor Orchestrator communication
# Protocol version 1.0.0

@0x1.2.3.4.5.6.7.8;  # Version 1 identifier

# Main message union for request/response pattern
struct ExtractorMessage {
  union {
    request @0 :ExtractRequest;
    response @1 :ExtractResponse;
    error @2 :ErrorResponse;
    heartbeat @3 :Heartbeat;
  }
}

# Request sent to extractor via stdin
struct ExtractRequest {
  requestId @0 :UInt64;        # Unique request ID for tracking
  projectPath @1 :Text;        # Path to project being analyzed
  language @2 :Text;           # Programming language
  config @3 :Text;             # JSON configuration
  timeoutMs @4 :UInt32;        # Timeout in milliseconds
  version @5 :Text;            # Protocol version
}

# Response received from extractor via stdout
struct ExtractResponse {
  requestId @0 :UInt64;        # Must match request ID
  success @1 :Bool;            # Whether extraction succeeded
  ir @2 :Data;                 # Serialized IR (Cap'n Proto format)
  metadata @3 :Text;           # JSON metadata (version, stats, etc.)
  processingTimeMs @4 :UInt32; # Time taken for extraction
}

# Error response for failed extractions
struct ErrorResponse {
  requestId @0 :UInt64;        # Must match request ID
  errorCode @1 :UInt32;        # Error code
  errorMessage @2 :Text;       # Human-readable error message
  errorDetails @3 :Text;       # Additional error details (JSON)
}

# Heartbeat for liveness checks
struct Heartbeat {
  timestamp @0 :UInt64;        # Unix timestamp
  extractorName @1 :Text;      # Name of extractor
  status @2 :Text;             # Status: "running", "idle", etc.
}
