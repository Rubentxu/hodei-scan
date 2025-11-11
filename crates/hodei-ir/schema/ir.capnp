# Cap'n Proto schema for hodei-scan IR v3.3
# Unique schema ID: 0xf0a1b2c3d4e5f601

@0xf0a1b2c3d4e5f601;

# ═══════════════════════════════════════════════════════════════════
# Core Types (atomic facts only per ADR-001)
# ═══════════════════════════════════════════════════════════════════

struct Confidence {
  value @0 :Float64;  # Range: [0.0, 1.0]
}

struct ProjectPath {
  canonical @0 :Text;  # Canonical absolute path
}

struct LineNumber {
  value @0 :UInt32;  # >= 1
}

struct ColumnNumber {
  value @0 :UInt32;  # >= 1
}

struct FlowId {
  uuid @0 :Data;  # 16 bytes (UUID)
}

struct SourceLocation {
  file @0 :ProjectPath;
  startLine @1 :LineNumber;
  startColumn @2 :ColumnNumber;
  endLine @3 :LineNumber;
  endColumn @4 :ColumnNumber;
}

# ═══════════════════════════════════════════════════════════════════
# Enums
# ═══════════════════════════════════════════════════════════════════

enum Severity {
  info @0;
  minor @1;
  major @2;
  critical @3;
  blocker @4;
}

enum Ecosystem {
  npm @0;
  cargo @1;
  maven @2;
  gradle @3;
  pypi @4;
  nuget @5;
  go @6;
  rubyGems @7;
  composer @8;
}

enum FactValueType {
  string @0;
  number @1;
  boolean @2;
  array @3;
  object @4;
}

struct FactValue {
  union {
    string @0 :Text;
    number @1 :Float64;
    boolean @2 :Bool;
    array @3 :List(FactValue);
    object @4 :List(KeyValuePair);
  }
}

struct KeyValuePair {
  key @0 :Text;
  value @1 :FactValue;
}

# ═══════════════════════════════════════════════════════════════════
# FactType (Union with 17 atomic variants + Custom)
# ═══════════════════════════════════════════════════════════════════

struct FactType {
  union {
    # SAST (6 variants)
    taintSource @0 :TaintSourceData;
    taintSink @1 :TaintSinkData;
    sanitization @2 :SanitizationData;
    unsafeCall @3 :UnsafeCallData;
    cryptographicOperation @4 :CryptographicOperationData;
    vulnerability @5 :VulnerabilityData;

    # Quality (4 variants)
    function @6 :FunctionData;
    variable @7 :VariableData;
    codeSmell @8 :CodeSmellData;
    complexityViolation @9 :ComplexityViolationData;

    # SCA (3 variants)
    dependency @10 :DependencyData;
    dependencyVulnerability @11 :DependencyVulnerabilityData;
    license @12 :LicenseData;

    # Coverage (4 variants)
    uncoveredLine @13 :UncoveredLineData;
    lowTestCoverage @14 :LowTestCoverageData;
    coverageStats @15 :CoverageStatsData;

    # Custom (for plugin-defined types)
    custom @16 :CustomFactType;
  }
}

# SAST Data Types
struct TaintSourceData {
  var @0 :Text;
  flowId @1 :FlowId;
  sourceType @2 :Text;
  confidence @3 :Confidence;
}

struct TaintSinkData {
  func @0 :Text;
  consumesFlow @1 :FlowId;
  category @2 :Text;
  severity @3 :Severity;
}

struct SanitizationData {
  method @0 :Text;
  sanitizesFlow @1 :FlowId;
  effective @2 :Bool;
  confidence @3 :Confidence;
}

struct UnsafeCallData {
  functionName @0 :Text;
  reason @1 :Text;
  severity @2 :Severity;
}

struct CryptographicOperationData {
  algorithm @0 :Text;
  keyLength @1 :UInt32;
  secure @2 :Bool;
  recommendation @3 :Text;
}

struct VulnerabilityData {
  cweId @0 :Text;
  owaspCategory @1 :Text;
  severity @2 :Severity;
  cvssScore @3 :Float32;
  description @4 :Text;
  confidence @5 :Confidence;
}

# Quality Data Types
struct FunctionData {
  name @0 :Text;
  complexity @1 :UInt32;
  linesOfCode @2 :UInt32;
}

struct VariableData {
  name @0 :Text;
  scope @1 :Text;
  varType @2 :Text;
}

struct CodeSmellData {
  smellType @0 :Text;
  severity @1 :Severity;
  message @2 :Text;
}

struct ComplexityViolationData {
  metric @0 :Text;
  actual @1 :UInt32;
  threshold @2 :UInt32;
}

# SCA Data Types
struct DependencyData {
  name @0 :Text;
  version @1 :Text;
  ecosystem @2 :Ecosystem;
}

struct DependencyVulnerabilityData {
  dependency @0 :Text;
  cveId @1 :Text;
  severity @2 :Severity;
  cvssScore @3 :Float32;
  description @4 :Text;
}

struct LicenseData {
  dependency @0 :Text;
  licenseType @1 :Text;
  compatible @2 :Bool;
}

# Coverage Data Types
struct UncoveredLineData {
  location @0 :SourceLocation;
  coverage @1 :Float32;
}

struct LowTestCoverageData {
  file @0 :ProjectPath;
  percentage @1 :Float32;
  totalLines @2 :UInt32;
  coveredLines @3 :UInt32;
}

struct CoverageStatsData {
  scope @0 :Text;
  path @1 :ProjectPath;
  lineCoverage @2 :Float32;
  branchCoverage @3 :Float32;
}

# Custom Data Type (for plugin-defined types)
struct CustomFactType {
  discriminant @0 :Text;  # Unique plugin identifier
  data @1 :List(KeyValuePair);  # Dynamic key-value pairs
}

# ═══════════════════════════════════════════════════════════════════
# Fact
# ═══════════════════════════════════════════════════════════════════

struct Provenance {
  extractor @0 :Text;
  version @1 :Text;
  confidence @2 :Confidence;
  extractedAt @3 :Int64;  # Unix timestamp (microseconds)
}

struct Fact {
  id @0 :Data;  # 16 bytes (UUID)
  factType @1 :FactType;
  location @2 :SourceLocation;
  provenance @3 :Provenance;
}

# ═══════════════════════════════════════════════════════════════════
# IR Container
# ═══════════════════════════════════════════════════════════════════

struct ProjectMetadata {
  name @0 :Text;
  version @1 :Text;
  rootPath @2 :Text;
  language @3 :Text;
  gitCommit @4 :Text;
  gitBranch @5 :Text;
}

struct AnalysisStats {
  totalFacts @0 :UInt32;
  extractorsUsed @1 :List(Text);
  duration @2 :UInt64;  # milliseconds
}

struct SchemaVersion {
  major @0 :UInt16;
  minor @1 :UInt16;
}

struct IntermediateRepresentation {
  analysisId @0 :Data;  # 16 bytes (UUID)
  timestamp @1 :Int64;  # Unix timestamp (microseconds)
  metadata @2 :ProjectMetadata;
  facts @3 :List(Fact);
  stats @4 :AnalysisStats;
  schemaVersion @5 :SchemaVersion;
}
