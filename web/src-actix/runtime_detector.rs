use crate::models::Runtime;
use anyhow::Result;
use chrono::Utc;
use log::*;
use std::process::Command;
use uuid::Uuid;

/// Detects all available runtimes on the system
pub fn detect_runtimes() -> Result<Vec<Runtime>> {
    let mut runtimes = Vec::new();

    // Detect Java
    if let Some(runtime) = detect_java() {
        runtimes.push(runtime);
    }

    // Detect .NET
    if let Some(runtime) = detect_dotnet() {
        runtimes.push(runtime);
    }

    // Detect Node.js
    if let Some(runtime) = detect_nodejs() {
        runtimes.push(runtime);
    }

    // Detect Python
    if let Some(runtime) = detect_python() {
        runtimes.push(runtime);
    }

    // Detect Go
    if let Some(runtime) = detect_go() {
        runtimes.push(runtime);
    }

    info!("Detected {} runtimes on the system", runtimes.len());
    Ok(runtimes)
}

fn detect_java() -> Option<Runtime> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "where", "java"]).output().ok()?
    } else {
        Command::new("which").arg("java").output().ok()?
    };

    if !output.status.success() {
        return None;
    }

    let executable_path = String::from_utf8_lossy(&output.stdout).trim().lines().next()?.to_string();

    // Get version
    let version_output = Command::new("java").arg("-version").output().ok()?;
    let version_str = String::from_utf8_lossy(&version_output.stderr);
    let version = extract_version_from_output(&version_str, r#"version "(.+?)""#).unwrap_or_else(|| "Unknown".to_string());

    info!("Detected Java {} at {}", version, executable_path);

    Some(Runtime {
        id: Uuid::new_v4().to_string(),
        name: "java".to_string(),
        display_name: "Java".to_string(),
        version,
        executable_path,
        runtime_type: "java".to_string(),
        detected_at: Utc::now().to_rfc3339(),
        is_available: true,
    })
}

fn detect_dotnet() -> Option<Runtime> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "where", "dotnet"]).output().ok()?
    } else {
        Command::new("which").arg("dotnet").output().ok()?
    };

    if !output.status.success() {
        return None;
    }

    let executable_path = String::from_utf8_lossy(&output.stdout).trim().lines().next()?.to_string();

    // Get version
    let version_output = Command::new("dotnet").arg("--version").output().ok()?;
    let version = String::from_utf8_lossy(&version_output.stdout).trim().to_string();

    info!("Detected .NET {} at {}", version, executable_path);

    Some(Runtime {
        id: Uuid::new_v4().to_string(),
        name: "dotnet".to_string(),
        display_name: ".NET".to_string(),
        version,
        executable_path,
        runtime_type: "dotnet".to_string(),
        detected_at: Utc::now().to_rfc3339(),
        is_available: true,
    })
}

fn detect_nodejs() -> Option<Runtime> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "where", "node"]).output().ok()?
    } else {
        Command::new("which").arg("node").output().ok()?
    };

    if !output.status.success() {
        return None;
    }

    let executable_path = String::from_utf8_lossy(&output.stdout).trim().lines().next()?.to_string();

    // Get version
    let version_output = Command::new("node").arg("--version").output().ok()?;
    let version = String::from_utf8_lossy(&version_output.stdout).trim().trim_start_matches('v').to_string();

    info!("Detected Node.js {} at {}", version, executable_path);

    Some(Runtime {
        id: Uuid::new_v4().to_string(),
        name: "node".to_string(),
        display_name: "Node.js".to_string(),
        version,
        executable_path,
        runtime_type: "nodejs".to_string(),
        detected_at: Utc::now().to_rfc3339(),
        is_available: true,
    })
}

fn detect_python() -> Option<Runtime> {
    // Try python3 first, then python
    let python_cmd = if Command::new("python3").arg("--version").output().is_ok() { "python3" } else { "python" };

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "where", python_cmd]).output().ok()?
    } else {
        Command::new("which").arg(python_cmd).output().ok()?
    };

    if !output.status.success() {
        return None;
    }

    let executable_path = String::from_utf8_lossy(&output.stdout).trim().lines().next()?.to_string();

    // Get version
    let version_output = Command::new(python_cmd).arg("--version").output().ok()?;
    let version_str = String::from_utf8_lossy(&version_output.stdout);
    let version = version_str.trim().trim_start_matches("Python ").to_string();

    info!("Detected Python {} at {}", version, executable_path);

    Some(Runtime {
        id: Uuid::new_v4().to_string(),
        name: python_cmd.to_string(),
        display_name: "Python".to_string(),
        version,
        executable_path,
        runtime_type: "python".to_string(),
        detected_at: Utc::now().to_rfc3339(),
        is_available: true,
    })
}

fn detect_go() -> Option<Runtime> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "where", "go"]).output().ok()?
    } else {
        Command::new("which").arg("go").output().ok()?
    };

    if !output.status.success() {
        return None;
    }

    let executable_path = String::from_utf8_lossy(&output.stdout).trim().lines().next()?.to_string();

    // Get version
    let version_output = Command::new("go").arg("version").output().ok()?;
    let version_str = String::from_utf8_lossy(&version_output.stdout);
    let version = extract_version_from_output(&version_str, r"go version go(.+?) ").unwrap_or_else(|| "Unknown".to_string());

    info!("Detected Go {} at {}", version, executable_path);

    Some(Runtime {
        id: Uuid::new_v4().to_string(),
        name: "go".to_string(),
        display_name: "Go".to_string(),
        version,
        executable_path,
        runtime_type: "go".to_string(),
        detected_at: Utc::now().to_rfc3339(),
        is_available: true,
    })
}

fn extract_version_from_output(output: &str, pattern: &str) -> Option<String> {
    let re = regex::Regex::new(pattern).ok()?;
    let captures = re.captures(output)?;
    Some(captures.get(1)?.as_str().to_string())
}
