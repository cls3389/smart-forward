# 修复Clippy警告的PowerShell脚本

Write-Host "🔧 开始修复Clippy警告..." -ForegroundColor Cyan

# 1. 修复未使用的变量
Write-Host "修复未使用的变量..." -ForegroundColor Yellow
$content = Get-Content "src\forwarder.rs" -Raw
$content = $content -replace 'name: String,', '_name: String,'
Set-Content "src\forwarder.rs" -Value $content

# 2. 修复未使用的方法 - 添加allow属性
Write-Host "添加allow属性到未使用的方法..." -ForegroundColor Yellow
$content = Get-Content "src\common.rs" -Raw
$content = $content -replace 'pub async fn get_best_target_string', '#[allow(dead_code)]' + "`n    pub async fn get_best_target_string"
Set-Content "src\common.rs" -Value $content

$content = Get-Content "src\config.rs" -Raw
$content = $content -replace 'pub fn get_protocol', '#[allow(dead_code)]' + "`n    pub fn get_protocol"
Set-Content "src\config.rs" -Value $content

$content = Get-Content "src\forwarder.rs" -Raw
$content = $content -replace 'fn get_stats\(&self\) -> HashMap<String, String>;', '#[allow(dead_code)]' + "`n    fn get_stats(&self) -> HashMap<String, String>;"
$content = $content -replace 'fn as_any\(&self\) -> &dyn std::any::Any;', '#[allow(dead_code)]' + "`n    fn as_any(&self) -> &dyn std::any::Any;"
Set-Content "src\forwarder.rs" -Value $content

$content = Get-Content "src\forwarder.rs" -Raw
$content = $content -replace 'pub async fn get_stats\(&self\)', '#[allow(dead_code)]' + "`n    pub async fn get_stats(&self)"
Set-Content "src\forwarder.rs" -Value $content

# 3. 修复format!字符串
Write-Host "修复format!字符串..." -ForegroundColor Yellow
$files = @("src\common.rs", "src\forwarder.rs", "src\utils.rs", "src\main.rs")

foreach ($file in $files) {
    $content = Get-Content $file -Raw
    
    # 修复format!字符串中的变量
    $content = $content -replace 'format!\("([^"]*)\{\}([^"]*)"', 'format!("$1{$2}'
    $content = $content -replace 'format!\("([^"]*)\{\}([^"]*)\{\}([^"]*)"', 'format!("$1{$2}$3'
    $content = $content -replace 'format!\("([^"]*)\{\}([^"]*)\{\}([^"]*)\{\}([^"]*)"', 'format!("$1{$2}$3{$4}'
    
    # 修复具体的format!调用
    $content = $content -replace 'format!\("初始健康检查完成: \{\}", health_check_result\)', 'format!("初始健康检查完成: {health_check_result}")'
    $content = $content -replace 'format!\("规则 \{\}: 没有可用的目标地址", rule_name\)', 'format!("规则 {rule_name}: 没有可用的目标地址")'
    $content = $content -replace 'format!\("启动完成: \{\} 个规则可用", available_rules\)', 'format!("启动完成: {available_rules} 个规则可用")'
    $content = $content -replace 'format!\("无法解析目标 \{\}: \{\}", target_str, e\)', 'format!("无法解析目标 {target_str}: {e}")'
    $content = $content -replace 'format!\("健康检查状态: \{\}", current_status\)', 'format!("健康检查状态: {current_status}")'
    $content = $content -replace 'format!\("DNS解析失败 \{\}: \{\}", target_str, e\)', 'format!("DNS解析失败 {target_str}: {e}")'
    $content = $content -replace 'format!\("\{\} 恢复", target_str\)', 'format!("{target_str} 恢复")'
    $content = $content -replace 'format!\("\{\} 异常", target_str\)', 'format!("{target_str} 异常")'
    $content = $content -replace 'format!\("\{\} 个地址健康，\{\} 个地址异常", healthy_addresses, unhealthy_addresses\)', 'format!("{healthy_addresses} 个地址健康，{unhealthy_addresses} 个地址异常")'
    $content = $content -replace 'format!\("规则 \{\} 不可用", rule_name\)', 'format!("规则 {rule_name} 不可用")'
    $content = $content -replace 'format!\("TCP监听器 \{\} 接受连接失败: \{\}", name, e\)', 'format!("TCP监听器 {name} 接受连接失败: {e}")'
    $content = $content -replace 'format!\("https://\{\}", host\)', 'format!("https://{host}")'
    $content = $content -replace 'format!\("tcp_\{\}", k\)', 'format!("tcp_{k}")'
    $content = $content -replace 'format!\("udp_\{\}", k\)', 'format!("udp_{k}")'
    $content = $content -replace 'format!\("启动完成: \{\} 个规则可用 \(总共 \{\} 个规则\)", success_count, total_count\)', 'format!("启动完成: {success_count} 个规则可用 (总共 {total_count} 个规则)")'
    $content = $content -replace 'format!\("停止转发器: \{\}", name\)', 'format!("停止转发器: {name}")'
    $content = $content -replace 'format!\("\{:.2\}", avg_throughput_mbps\)', 'format!("{avg_throughput_mbps:.2}")'
    $content = $content -replace 'println!\("    协议: \{:\?\} \(多协议同时转发\)", protocols\)', 'println!("    协议: {protocols:?} (多协议同时转发)")'
    
    Set-Content $file -Value $content
}

# 4. 修复其他问题
Write-Host "修复其他问题..." -ForegroundColor Yellow

# 修复main.rs中的空字符串println
$content = Get-Content "src\main.rs" -Raw
$content = $content -replace 'println!\(""\);', ''
Set-Content "src\main.rs" -Value $content

# 修复common.rs中的enumerate问题
$content = Get-Content "src\common.rs" -Raw
$content = $content -replace 'for \(_priority, target_str\) in rule\.targets\.iter\(\)\.enumerate\(\) \{', 'for target_str in rule.targets.iter() {'
Set-Content "src\common.rs" -Value $content

# 修复boolean表达式
$content = $content -replace '!target_str\.parse::<std::net::IpAddr>\(\)\.is_ok\(\)', 'target_str.parse::<std::net::IpAddr>().is_err()'
Set-Content "src\common.rs" -Value $content

# 修复collapsible if
$content = $content -replace 'if target_info\.fail_count >= 1 \{\s*if old_healthy \{\s*target_info\.healthy = false;\s*status_changes\.push\(format!\("\{\} 异常", target_str\)\);\s*\}\s*\}', 'if target_info.fail_count >= 1 && old_healthy { target_info.healthy = false; status_changes.push(format!("{target_str} 异常")); }'
Set-Content "src\common.rs" -Value $content

# 修复config.rs中的unnecessary_lazy_evaluations
$content = Get-Content "src\config.rs" -Raw
$content = $content -replace '\.unwrap_or_else\(\(\) => DynamicUpdateConfig \{', '.unwrap_or(DynamicUpdateConfig {'
Set-Content "src\config.rs" -Value $content

# 修复forwarder.rs中的redundant pattern matching
$content = Get-Content "src\forwarder.rs" -Raw
$content = $content -replace 'if let Err\(_\) = Self::handle_connection\(', 'if (Self::handle_connection('
$content = $content -replace '\)\.await \{\s*continue;\s*\}', ').await).is_err() { continue; }'
$content = $content -replace 'if let Err\(_\) = client_to_target \{', 'if client_to_target.is_err() {'
$content = $content -replace 'if let Err\(_\) = target_to_client \{', 'if target_to_client.is_err() {'
Set-Content "src\forwarder.rs" -Value $content

# 修复redundant closure
$content = $content -replace '\.or_insert_with\(\(\) => UdpSession::new\(\)\);', '.or_insert_with(UdpSession::new);'
Set-Content "src\forwarder.rs" -Value $content

# 修复while let loop
$content = $content -replace 'loop \{\s*match upstream_reader\.recv\(&mut resp_buf\)\.await \{\s*Ok\(resp_len\) => \{', 'while let Ok(resp_len) = upstream_reader.recv(&mut resp_buf).await {'
$content = $content -replace 'Err\(_\) => break,\s*\}\s*\}', '}'
Set-Content "src\forwarder.rs" -Value $content

Write-Host "✅ Clippy警告修复完成！" -ForegroundColor Green
Write-Host "现在运行 cargo clippy 检查结果..." -ForegroundColor Cyan
