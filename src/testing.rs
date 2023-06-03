/// Parsing and printing test results
/// from Unity. Unity outputs an XML file
/// that summarizes test results.
///
/// The basic structure is `<test-run>` as the
/// root element, then composed of `<test-suite>`
/// elements, which may be composed of more
/// `<test-suite>` elements, and finally
/// there are <test-case>` elements which give the results
/// of a single test case.

use quick_xml::de;
use serde::Deserialize;
use colored::Colorize;
use std::{fs::File, io::BufReader, fmt::Display, path::PathBuf};
use crate::display::*;


#[derive(Debug, Deserialize)]
pub struct TestSummary {
    #[serde(rename="$value")]
    test_suites: Vec<TestSuite>,
}
impl Display for TestSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
               self.test_suites.iter()
               .map(|suite| suite.to_string())
               .collect::<Vec<String>>().join("\n"))
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all="kebab-case")]
enum TestDetail {
    TestCase(TestCase),
    TestSuite(TestSuite),
    Failure(FailureInfo),
    Output(String), // Console output
    Properties, // Not much useful info here
    Reason(FailureInfo),
}
impl Display for TestDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            // Recurse
            TestDetail::TestSuite(suite) => {
                suite.to_string()
            },
            TestDetail::TestCase(case) => {
                case.to_string()
            },
            TestDetail::Failure(failure) => {
                failure.to_string()
            },
            TestDetail::Output(output) => {
                output.to_string()
            },
            TestDetail::Properties => {
                "[Properties]".to_string()
            },
            TestDetail::Reason(reason) => {
                reason.to_string()
            }
        };
        write!(f, "{}", msg)
    }
}


#[derive(Debug, Deserialize, PartialEq)]
enum TestResult {
    Failed,
    Passed,
    Skipped,
}
impl Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            TestResult::Failed => red("𐄂"),
            TestResult::Passed => green("✓"),
            TestResult::Skipped => muted("-"),
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct FailureInfo {
    #[serde(rename="$value")]
    details: Vec<FailureDetail>,
}
impl Display for FailureInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            self.details.iter()
                .map(|d| d.to_string())
                .collect::<Vec<String>>()
                .join("\n"))
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all="kebab-case")]
enum FailureDetail {
    Message(String),
    StackTrace(String),
}
impl Display for FailureDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            FailureDetail::Message(msg) => red(msg),
            FailureDetail::StackTrace(msg) => msg.truecolor(157, 174, 179),
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct TestCase {
    #[serde(rename="@name")]
    name: String,

    #[serde(rename="@result")]
    result: TestResult,

    #[serde(rename="$value")]
    details: Vec<TestDetail>
}
impl Display for TestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lines = vec![];
        let failed = self.result == TestResult::Failed;
        if failed {
            lines.push("".to_string()); // Empty line
        }

        lines.push(format!("{} {}", self.result, if failed {
            self.name.bold()
        } else {
            self.name.normal()
        }));

        for detail in &self.details {
            match detail {
                TestDetail::Properties => continue,
                TestDetail::Output(_) => {
                    // Don't print output if the test passed
                    if self.result == TestResult::Passed {
                        continue
                    } else {
                        let repr = detail.to_string();
                        if repr.is_empty() { continue }
                        lines.push(indent(&repr));
                    }
                },
                _ => {
                    let repr = detail.to_string();
                    if repr.is_empty() { continue }
                    lines.push(indent(&repr));
                }
            }
        }
        write!(f, "{}", lines.join("\n"))
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct TestSuite {
    #[serde(rename="@type")]
    kind: String,

    #[serde(rename="@name")]
    name: String,

    #[serde(rename="@failed")]
    failed: usize,

    #[serde(rename="@passed")]
    passed: usize,

    #[serde(rename="@skipped")]
    skipped: usize,

    #[serde(rename="@total")]
    total: usize,

    #[serde(rename="$value")]
    details: Vec<TestDetail>
}
impl Display for TestSuite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lines: Vec<String> = vec![];
        let name = if self.failed > 0 {
            on_red(&format!(" {} ", self.name))
        } else if self.passed == self.total {
            on_green(&format!(" {} ", self.name))
        } else {
            muted(&self.name).bold()
        };
        lines.push(format!("{} {} {} {}",
            name,
            green(&self.passed.to_string()),
            red(&self.failed.to_string()),
            muted(&self.skipped.to_string())));
        for detail in &self.details {
            if *detail == TestDetail::Properties { continue }
            let repr = detail.to_string();
            if repr.is_empty() { continue }
            lines.push(indent(&repr));
        }
        write!(f, "{}", lines.join("\n"))
    }
}

pub fn load_test_results(results_path: &PathBuf) -> TestSummary {
    let file = File::open(results_path).unwrap();
    let buf_reader = BufReader::new(file);
    let results: TestSummary = de::from_reader(buf_reader).unwrap();
    results
}
