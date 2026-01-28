# PowerShell script to fix DaemonEvent usages in ws_test.rs

$file = "keyrx_daemon\src\web\ws_test.rs"
$content = Get-Content $file -Raw

# Fix creation patterns
$content = $content -replace 'DaemonEvent::State\((DaemonState \{[^}]+\})\)', 'DaemonEvent::State { data: $1, sequence: 1 }'
$content = $content -replace 'DaemonEvent::KeyEvent\((KeyEventData \{[^}]+\})\)', 'DaemonEvent::KeyEvent { data: $1, sequence: 1 }'
$content = $content -replace 'DaemonEvent::Latency\((LatencyStats \{[^}]+\})\)', 'DaemonEvent::Latency { data: $1, sequence: 1 }'

# Fix match patterns
$content = $content -replace 'DaemonEvent::State\(([a-z_]+)\)', 'DaemonEvent::State { data: $1, sequence: _ }'
$content = $content -replace 'DaemonEvent::KeyEvent\(([a-z_]+)\)', 'DaemonEvent::KeyEvent { data: $1, sequence: _ }'
$content = $content -replace 'DaemonEvent::Latency\(([a-z_]+)\)', 'DaemonEvent::Latency { data: $1, sequence: _ }'

# Fix matches! patterns
$content = $content -replace 'matches!\(([^,]+), DaemonEvent::State\(_\)\)', 'matches!($1, DaemonEvent::State { .. })'
$content = $content -replace 'matches!\(([^,]+), DaemonEvent::KeyEvent\(_\)\)', 'matches!($1, DaemonEvent::KeyEvent { .. })'
$content = $content -replace 'matches!\(([^,]+), DaemonEvent::Latency\(_\)\)', 'matches!($1, DaemonEvent::Latency { .. })'

# Fix JSON assertions for payload -> data
$content = $content -replace 'parsed\["payload"\]', 'parsed'

$content | Set-Content $file
Write-Host "Fixed ws_test.rs"
