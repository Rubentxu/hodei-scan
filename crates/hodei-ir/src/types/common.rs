use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum Severity {
    Info,
    Minor,
    Major,
    Critical,
    Blocker,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VariableName(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionName(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ecosystem {
    Npm,
    Cargo,
    Maven,
    Gradle,
    PyPI,
    NuGet,
    Go,
    RubyGems,
    Composer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CveId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CoveragePercentage(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtractorId {
    TreeSitter,
    OxcParser,
    SemgrepTaint,
    DataFlowAnalyzer,
    SymbolicExecutor,
    CargoAudit,
    NpmAudit,
    TrivyScanner,
    JaCoCoParser,
    LcovParser,
    CoberturaParser,
    Custom,
}
