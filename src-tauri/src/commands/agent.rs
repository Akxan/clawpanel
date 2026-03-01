/// Agent 管理命令 — 调用 openclaw CLI 实现增删改查
use serde_json::Value;
use std::process::Command;
use std::fs;
use std::io::Write;

/// 获取 agent 列表
#[tauri::command]
pub fn list_agents() -> Result<Value, String> {
    let output = Command::new("openclaw")
        .args(["agents", "list", "--json"])
        .output()
        .map_err(|e| format!("执行失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("获取 Agent 列表失败: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| format!("解析 JSON 失败: {e}"))
}

/// 创建新 agent
#[tauri::command]
pub fn add_agent(name: String, model: String, workspace: Option<String>) -> Result<Value, String> {
    let ws = match workspace {
        Some(ref w) if !w.is_empty() => std::path::PathBuf::from(w),
        _ => super::openclaw_dir()
            .join("agents")
            .join(&name)
            .join("workspace"),
    };

    let mut args = vec![
        "agents".to_string(),
        "add".to_string(),
        name.clone(),
        "--non-interactive".to_string(),
        "--workspace".to_string(),
        ws.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    if !model.is_empty() {
        args.push("--model".to_string());
        args.push(model);
    }

    let output = Command::new("openclaw")
        .args(&args)
        .output()
        .map_err(|e| format!("执行失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("创建 Agent 失败: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).unwrap_or(Value::String("ok".into()));
    // 返回最新列表
    list_agents()
}

/// 删除 agent
#[tauri::command]
pub fn delete_agent(id: String) -> Result<String, String> {
    if id == "main" {
        return Err("不能删除默认 Agent".into());
    }

    let output = Command::new("openclaw")
        .args(["agents", "delete", &id])
        .output()
        .map_err(|e| format!("执行失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("删除 Agent 失败: {stderr}"));
    }

    Ok("已删除".into())
}

/// 更新 agent 身份信息
#[tauri::command]
pub fn update_agent_identity(
    id: String,
    name: Option<String>,
    emoji: Option<String>,
) -> Result<String, String> {
    let mut args = vec![
        "agents".to_string(),
        "set-identity".to_string(),
        "--agent".to_string(),
        id,
        "--json".to_string(),
    ];

    if let Some(n) = name {
        if !n.is_empty() {
            args.push("--name".to_string());
            args.push(n);
        }
    }
    if let Some(e) = emoji {
        if !e.is_empty() {
            args.push("--emoji".to_string());
            args.push(e);
        }
    }

    let output = Command::new("openclaw")
        .args(&args)
        .output()
        .map_err(|e| format!("执行失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("更新失败: {stderr}"));
    }

    Ok("已更新".into())
}

/// 备份 agent 数据（agent 配置 + 会话记录）打包为 zip
#[tauri::command]
pub fn backup_agent(id: String) -> Result<String, String> {
    let agent_dir = super::openclaw_dir().join("agents").join(&id);
    if !agent_dir.exists() {
        return Err(format!("Agent「{id}」数据目录不存在"));
    }

    let tmp_dir = std::env::temp_dir();
    let now = chrono::Local::now();
    let zip_name = format!("agent-{}-{}.zip", id, now.format("%Y%m%d-%H%M%S"));
    let zip_path = tmp_dir.join(&zip_name);

    let file = fs::File::create(&zip_path)
        .map_err(|e| format!("创建 zip 失败: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    collect_dir_to_zip(&agent_dir, &agent_dir, &mut zip, options)?;

    zip.finish().map_err(|e| format!("完成 zip 失败: {e}"))?;
    Ok(zip_path.to_string_lossy().to_string())
}

fn collect_dir_to_zip(
    base: &std::path::Path,
    dir: &std::path::Path,
    zip: &mut zip::ZipWriter<fs::File>,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("读取目录失败: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(base)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        if path.is_dir() {
            collect_dir_to_zip(base, &path, zip, options)?;
        } else {
            let content = fs::read(&path)
                .map_err(|e| format!("读取 {rel} 失败: {e}"))?;
            zip.start_file(&rel, options)
                .map_err(|e| format!("写入 zip 失败: {e}"))?;
            zip.write_all(&content)
                .map_err(|e| format!("写入内容失败: {e}"))?;
        }
    }
    Ok(())
}
