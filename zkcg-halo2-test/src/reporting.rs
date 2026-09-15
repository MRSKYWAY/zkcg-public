#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub phase: &'static str,
    pub category: &'static str,
    pub test_name: String,
    pub outcome: &'static str,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EvidenceSummary {
    records: Vec<EvidenceRecord>,
}

impl EvidenceSummary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, record: EvidenceRecord) {
        self.records.push(record);
    }

    pub fn records(&self) -> &[EvidenceRecord] {
        &self.records
    }

    pub fn total(&self) -> usize {
        self.records.len()
    }

    pub fn passed(&self) -> usize {
        self.records.iter().filter(|r| r.outcome == "pass").count()
    }

    pub fn failed(&self) -> usize {
        self.records.iter().filter(|r| r.outcome == "fail").count()
    }

    pub fn is_clean(&self) -> bool {
        self.failed() == 0
    }
}
