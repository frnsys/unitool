mod testing;
pub mod display;

use anyhow::Result;
use clap::ValueEnum;
use std::{process::Command, str, fs, path::PathBuf, collections::HashSet};
use testing::{TestSummary, load_test_results};

const UNITY_DIR: &str = "/opt/Unity/";
const TEST_RESULTS_PATH: &str = "/tmp/unity-test-results.xml";

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum TestMode {
    EditMode,
    PlayMode,
}

type CompileErrors = HashSet<String>;

/// Compile the project, returning any errors
pub fn compile(project_path: &PathBuf) -> Result<CompileErrors> {
    run_unity(project_path, vec!["-quit"])
}

/// Test the project, with optional filters.
/// For what filters work, see:
/// <https://docs.unity3d.com/Packages/com.unity.test-framework@1.1/manual/reference-command-line.html>
pub fn test(project_path: &PathBuf, mode: TestMode, assemblies: &str, filters: Option<String>) -> Result<(CompileErrors, Option<TestSummary>)> {
    let platform = match mode {
        TestMode::EditMode => "EditMode",
        TestMode::PlayMode => "PlayMode",
    };

    let filters = filters.unwrap_or("".to_string());
    let mut args = vec![
      "-runTests",
      "-testPlatform", platform,
      "-testResults", TEST_RESULTS_PATH,
      "-testFilter", &filters,
      "-assemblyNames", assemblies,
    ];

    // Edit mode tests need to run synchronously
    // or they lock up Unity in batchmode
    if mode == TestMode::EditMode {
        args.push("-runSynchronously");
    }

    let errs = run_unity(project_path, args)?;

    if errs.is_empty() {
        let results = load_test_results(&TEST_RESULTS_PATH.into());
        Ok((errs, Some(results)))
    } else {
        Ok((errs, None))
    }
}

/// Find the path to the most recent Unity Editor binary.
fn find_unity_path() -> Result<PathBuf> {
    let mut cands = fs::read_dir(UNITY_DIR)?
        .map(|dir| dir.unwrap().path())
        .collect::<Vec<PathBuf>>();
    cands.sort();

    let dir = &cands[0];
    let path = PathBuf::from(dir).join("Editor/Unity");
    Ok(path)
}

/// Run Unity in headless mode with the provided commands.
fn run_unity(project_path: &PathBuf, args: Vec<&str>) -> Result<HashSet<String>> {
    let path = find_unity_path()?;
    let mut cmd = Command::new(path);

    cmd.args([vec![
        "-batchmode",       // run headless
        "-logfile", "-",    // log to stdout
        "-projectPath", project_path.to_str().unwrap(),
    ], args].concat());

    let output = cmd.output()?;
    let output = str::from_utf8(&output.stdout)?;
    let errors: HashSet<String> = output.lines()
        .filter(|line| line.contains("error CS"))
        .map(|line| line.into())
        .collect();

    Ok(errors)
}
