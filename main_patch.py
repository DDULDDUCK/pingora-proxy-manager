import sys

with open('backend/src/main.rs', 'r') as f:
    lines = f.readlines()

new_lines = []
skip = False
for i, line in enumerate(lines):
    if '// 1. 로깅 초기화' in line:
        new_lines.append(line)
        new_lines.append('    let _guard = bootstrap::logging::init_logging();\n')
        skip = True
        continue
    if '// 메트릭 레코더 초기화' in line:
        skip = False
    if skip and 'tracing_subscriber' in line:
        continue
    if skip and 'let file_appender' in line:
        continue
    if skip and 'let (non_blocking' in line:
        continue
    if skip and '.with(' in line:
        continue
    if skip and '.init();' in line:
        continue
    if skip and '.json()' in line:
        continue
    if skip and '.with_writer' in line:
        continue
    if skip and '.with_filter' in line:
        continue
    if skip and ')' in line and i < 43: # rough estimate
        continue
    
    # DB section
    if '// 2. DB 초기화' in line:
        new_lines.append(line)
        new_lines.append('        let db_url = "sqlite:data/data.db?mode=rwc";\n')
        new_lines.append('        let pool = bootstrap::db::init_db(db_url).await?;\n')
        skip_db = True
        continue
    
    if '// 3. 초기 상태 로드' in line:
        skip_db = False
    
    if 'skip_db' in locals() and skip_db:
        if 'let db_url' in line or 'let pool' in line or 'admin_exists' in line or 'if !admin_exists' in line or 'let hash' in line or 'db::create_user' in line or 'tracing::info!("👤' in line or '}' in line:
             # This is a bit risky, let's just use exact match for DB block
             pass
        else:
             new_lines.append(line)
        continue

    # Metric handle
    if 'let recorder_handle = PrometheusBuilder::new()' in line:
        new_lines.append('    let recorder_handle = bootstrap::metrics::init_metrics()?;\n')
        continue
    if '.install_recorder()' in line or '.expect("failed to install Prometheus recorder")' in line:
        continue

    if not skip:
        new_lines.append(line)

# Let's try a simpler approach with exact string replacement for the blocks
