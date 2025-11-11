@0x1000000000000000;  # Magic number for Cap'n Proto

# Enums for different fact types and categories

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

enum FactTypeDiscriminant {
  taintSource @0;
  taintSink @1;
  sanitization @2;
  unsafeCall @3;
  cryptographicOperation @4;
  vulnerability @5;
  function @6;
  variable @7;
  codeSmell @8;
  complexityViolation @9;
  dependency @10;
  dependencyVulnerability @11;
  license @12;
  uncoveredLine @13;
  lowTestCoverage @14;
  coverageStats @15;
}

enum ExtractorId {
  treeSitter @0;
  oxcParser @1;
  semgrepTaint @2;
  dataFlowAnalyzer @3;
  symbolicExecutor @4;
  cargoAudit @5;
  npmAudit @6;
  trivyScanner @7;
  jaCoCoParser @8;
  lcovParser @9;
  coberturaParser @10;
  custom @11;
}

# Data structures

struct Uuid {
  bytes @0 :Data;
}

struct Confidence {
  value @0 :Float64;
}

struct CveId {
  id @0 :UInt32;
}

struct CoveragePercentage {
  value @0 :Float32;
}

struct VariableName {
  name @0 :Text;
}

struct FunctionName {
  name @0 :Text;
}

struct FlowId {
  id @0 :Uuid;
}

struct LineNumber {
  value @0 :UInt32;
}

struct ColumnNumber {
  value @0 :UInt32;
}

struct ProjectPath {
  path @0 :Text;
}

# Source location
struct SourceLocation {
  file @0 :ProjectPath;
  startLine @1 :LineNumber;
  startColumn @2 :ColumnNumber;
  endLine @3 :LineNumber;
  endColumn @4 :ColumnNumber;
}

# Provenance information
struct Provenance {
  extractor @0 :ExtractorId;
  version @1 :Text;
  confidence @2 :Confidence;
  extractedAt @3 :Int64;  # Unix timestamp
}

# SAST Fact Types

struct TaintSource {
  var @0 :VariableName;
  flowId @1 :FlowId;
  sourceType @2 :Text;
  confidence @3 :Confidence;
}

struct TaintSink {
  func @4 :FunctionName;
  consumesFlow @5 :FlowId;
  category @6 :Text;
  severity @7 :Severity;
}

struct Sanitization {
  method @8 :Text;
  sanitizesFlow @9 :FlowId;
  effective @10 :Bool;
  confidence @11 :Confidence;
}

struct UnsafeCall {
  functionName @12 :FunctionName;
  reason @13 :Text;
  severity @14 :Severity;
}

struct CryptographicOperation {
  algorithm @15 :Text;
  keyLength @16 :UInt32;
  secure @17 :Bool;
  recommendation @18 :Text;
}

struct Vulnerability {
  cweId @19 :Text;
  owaspCategory @20 :Text;
  severity @21 :Severity;
  cvssScore @22 :Float32;
  description @23 :Text;
  confidence @24 :Confidence;
}

# Quality Fact Types

struct Function {
  name @25 :FunctionName;
  complexity @26 :UInt32;
  linesOfCode @27 :UInt32;
}

struct Variable {
  name @28 :VariableName;
  scope @29 :Text;
  varType @30 :Text;
}

struct CodeSmell {
  smellType @31 :Text;
  severity @32 :Severity;
  message @33 :Text;
}

struct ComplexityViolation {
  metric @34 :Text;
  actual @35 :UInt32;
  threshold @36 :UInt32;
}

# SCA Fact Types

struct Dependency {
  name @37 :Text;
  version @38 :Text;
  ecosystem @39 :Ecosystem;
}

struct DependencyVulnerability {
  dependency @40 :Text;
  cveId @41 :Text;
  severity @42 :Severity;
  cvssScore @43 :Float32;
  description @44 :Text;
}

struct License {
  dependency @45 :Text;
  licenseType @46 :Text;
  compatible @47 :Bool;
}

# Coverage Fact Types

struct UncoveredLine {
  location @48 :SourceLocation;
  coverage @49 :Text;
}

struct LowTestCoverage {
  file @50 :ProjectPath;
  percentage @51 :UInt32;
  totalLines @52 :UInt32;
  coveredLines @53 :UInt32;
}

struct CoverageStats {
  scope @54 :Text;
  path @55 :ProjectPath;
  lineCoverage @56 :UInt32;
  branchCoverage @57 :UInt32;
}

# Union for all fact types
struct FactData {
  discriminant @0 :FactTypeDiscriminant;

  taintSource @1 :TaintSource;
  taintSink @2 :TaintSink;
  sanitization @3 :Sanitization;
  unsafeCall @4 :UnsafeCall;
  cryptographicOperation @5 :CryptographicOperation;
  vulnerability @6 :Vulnerability;
  function @7 :Function;
  variable @8 :Variable;
  codeSmell @9 :CodeSmell;
  complexityViolation @10 :ComplexityViolation;
  dependency @11 :Dependency;
  dependencyVulnerability @12 :DependencyVulnerability;
  license @13 :License;
  uncoveredLine @14 :UncoveredLine;
  lowTestCoverage @15 :LowTestCoverage;
  coverageStats @16 :CoverageStats;
}

# The main Fact structure
struct Fact {
  id @0 :Uuid;
  data @1 :FactData;
  location @2 :SourceLocation;
  provenance @3 :Provenance;
}

# Project metadata
struct ProjectMetadata {
  name @0 :Text;
  version @1 :Text;
  rootPath @2 :ProjectPath;
}

# The complete IR structure
struct IR {
  facts @0 :List(Fact);
  metadata @1 :ProjectMetadata;
  schemaVersion @2 :Text;
}
